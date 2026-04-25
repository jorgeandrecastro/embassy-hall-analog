// Copyright (C) 2026 Jorge Andre Castro
// GPL-2.0-or-later
//! # embassy-hall-analog
//!
//! Driver async `no_std` minimaliste pour le **capteur à effet Hall linéaire analogique**
//! OPEN-SMART (compatible 49E / SS49E) sur microcontrôleur RP2040 et RP235x,
//! basé sur le framework [Embassy](https://embassy.dev), ce n'est pas développé par l'équipe officielle attention!!.
//!
//! ## Description du composant
//!
//! Ce module capteur repose sur un capteur à effet Hall linéaire (type 49E) qui délivre
//! une tension analogique proportionnelle à l'intensité du champ magnétique environnant.
//! En l'absence de champ magnétique, la sortie se stabilise autour de **VCC / 2** (~1,65 V
//! sous 3,3 V). Un pôle Sud rapproche la tension de VCC ; un pôle Nord la tire vers GND.
//!
//! ## Schéma de câblage
//!
//! ```text
//! Module Hall OPEN-SMART        RP2040 / RP235x
//! ─────────────────────────     ───────────────
//!        VCC  ────────────────── 3.3V
//!        GND  ────────────────── GND
//!        AO   ────────────────── GP26 (ADC0)
//! ```
//!
//! > **Note :** La broche AO (Analog Output) est connectée directement à la broche ADC
//! > du microcontrôleur. Aucune résistance externe n'est nécessaire : le module embarque
//! > déjà la polarisation interne.
//!
//! ## Features disponibles
//!
//! | Feature   | Cible                        | Défaut |
//! |-----------|------------------------------|--------|
//! | `rp2040`  | Raspberry Pi Pico 1 (RP2040) | ✓      |
//! | `rp235xa` | Pico 2 A-step (RP2350A)      |        |
//! | `rp235xb` | Pico 2 B-step (RP2350B)      |        |
//!
//! Ces features sont **mutuellement exclusives**. Le build échouera si zéro ou plusieurs
//! sont activées simultanément.
//!
//! ## Exemple d'utilisation
//!
//! ```rust,no_run
//! #![no_std]
//! #![no_main]
//!
//! use embassy_executor::Spawner;
//! use embassy_rp::adc::{Adc, Channel, Config as AdcConfig};
//! use embassy_rp::bind_interrupts;
//! use embassy_rp::adc::InterruptHandler;
//! use embassy_hall_analog::HallAnalog;
//!
//! bind_interrupts!(struct Irqs {
//!     ADC_IRQ_FIFO => InterruptHandler;
//! });
//!
//! #[embassy_executor::main]
//! async fn main(_spawner: Spawner) {
//!     let p = embassy_rp::init(Default::default());
//!
//!     let adc = Adc::new(p.ADC, Irqs, AdcConfig::default());
//!     let channel = Channel::new_pin(p.PIN_26, embassy_rp::gpio::Pull::None);
//!
//!     let mut sensor = HallAnalog::new(adc, channel);
//!
//!     loop {
//!         let raw = sensor.read_raw().await;
//!         // ~2048 (RP2040) / ~8192 (RP235x) → pas de champ
//!         // > repos → pôle Sud  |  < repos → pôle Nord
//!         let _ = raw;
//!     }
//! }
//! ```
//!
//! ## Calcul de tension et de champ magnétique
//!
//! La valeur brute ADC peut être convertie en tension :
//!
//! ```text
//! V = raw × 3.3 / MAX     (MAX = 4095 sur RP2040, 16383 sur RP235x)
//! ```
//!
//! La déviation par rapport à la tension de repos (~VCC/2) indique l'intensité et la polarité :
//!
//! ```text
//! ΔV = V - 1.65          (positif → pôle Sud, négatif → pôle Nord)
//! ```
//!
//! Pour le capteur 49E, la sensibilité typique est de **1,4 mV/Gauss** à 5 V
//! (environ **0,92 mV/Gauss** à 3,3 V) :
//!
//! ```text
//! B (Gauss) = ΔV / 0.00092
//! ```
//!
//! ## Caractéristiques
//!
//! | Paramètre              | Valeur                                  |
//! |------------------------|-----------------------------------------|
//! | Tension d'alimentation | 3,3 V (RP2040 / RP235x)                 |
//! | Résolution ADC         | 12 bits (0–4095) / 14 bits sur RP235x  |
//! | Sortie repos           | ~VCC / 2 (~1,65 V sous 3,3 V)           |
//! | Sortie pôle Sud        | Augmente vers VCC                       |
//! | Sortie pôle Nord       | Diminue vers GND                        |
//! | Sensibilité typique    | ~0,92 mV/Gauss @ 3,3 V (capteur 49E)   |
//! | Interface              | Analogique (AO) — lecture ADC directe   |
//!
//! ## `no_std`
//!
//! Cette crate ne dépend pas de la bibliothèque standard et est conçue pour
//! tourner sur des microcontrôleurs bare-metal avec le runtime Embassy.

#![no_std]
#![forbid(unsafe_code)]

// ---------------------------------------------------------------------------
// Garde de compilation : exactement une feature cible doit être activée.
// Le build.rs émet une erreur explicite si zéro ou plusieurs sont présentes,
// mais ce cfg guard est une sécurité supplémentaire au niveau de la crate.
// ---------------------------------------------------------------------------
#[cfg(not(any(feature = "rp2040", feature = "rp235xa", feature = "rp235xb")))]
compile_error!(
    "embassy-hall-analog : aucune cible sélectionnée.\n\
     Activez exactement une feature parmi : rp2040, rp235xa, rp235xb."
);

#[cfg(all(feature = "rp2040", feature = "rp235xa"))]
compile_error!(
    "embassy-hall-analog : features `rp2040` et `rp235xa` activées simultanément.\n\
     Ces features sont mutuellement exclusives."
);

#[cfg(all(feature = "rp2040", feature = "rp235xb"))]
compile_error!(
    "embassy-hall-analog : features `rp2040` et `rp235xb` activées simultanément.\n\
     Ces features sont mutuellement exclusives."
);

#[cfg(all(feature = "rp235xa", feature = "rp235xb"))]
compile_error!(
    "embassy-hall-analog : features `rp235xa` et `rp235xb` activées simultanément.\n\
     Ces features sont mutuellement exclusives."
);

use embassy_rp::adc::{Adc, Async, Channel};

// ---------------------------------------------------------------------------
// Constantes publiques nommées explicitement (rétrocompatibilité)
// ---------------------------------------------------------------------------

/// Valeur ADC correspondant à l'absence de champ magnétique **12 bits, RP2040**.
///
/// En pratique, cette valeur peut légèrement varier d'un module à l'autre.
/// Utilisez [`HallAnalog::read_raw`] pour calibrer votre point zéro réel.
pub const ZERO_FIELD_RAW_12BIT: u16 = 2048;

/// Valeur ADC correspondant à l'absence de champ magnétique - **14 bits, RP235x**.
pub const ZERO_FIELD_RAW_14BIT: u16 = 8192;

// ---------------------------------------------------------------------------
// Constante interne sélectionnée automatiquement selon la feature active.
// Utilisée par `read_polarity` pour éviter toute duplication de logique.
// ---------------------------------------------------------------------------

/// Point de repos ADC sélectionné automatiquement selon la cible compilée.
///
/// - `rp2040`          → [`ZERO_FIELD_RAW_12BIT`] (2048)
/// - `rp235xa` / `rp235xb` → [`ZERO_FIELD_RAW_14BIT`] (8192)
#[cfg(feature = "rp2040")]
const ZERO_FIELD_RAW: u16 = ZERO_FIELD_RAW_12BIT;

#[cfg(any(feature = "rp235xa", feature = "rp235xb"))]
const ZERO_FIELD_RAW: u16 = ZERO_FIELD_RAW_14BIT;

// ---------------------------------------------------------------------------
// Types publics
// ---------------------------------------------------------------------------

/// Driver pour le capteur à effet Hall linéaire analogique OPEN-SMART
/// via l'ADC du RP2040 / RP235x.
///
/// Ce driver encapsule un canal ADC Embassy et fournit une lecture asynchrone
/// de la valeur brute du capteur. La valeur brute est centrée autour de
/// [`ZERO_FIELD_RAW_12BIT`] (RP2040) ou [`ZERO_FIELD_RAW_14BIT`] (RP235x)
/// en l'absence de champ magnétique.
///
/// # Exemple minimal
///
/// ```rust,no_run
/// # use embassy_rp::adc::{Adc, Channel};
/// # use embassy_hall_analog::HallAnalog;
/// // let mut sensor = HallAnalog::new(adc, channel);
/// // let raw: u16 = sensor.read_raw().await;
/// ```
pub struct HallAnalog<'d> {
    adc: Adc<'d, Async>,
    channel: Channel<'d>,
}

/// Polarité du champ magnétique détecté.
///
/// La détection est basée sur la déviation de la valeur brute ADC
/// par rapport au point de repos (mi-échelle), avec une zone morte configurable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MagneticPolarity {
    /// Pôle Sud détecté la tension monte au-dessus de VCC/2.
    SouthPole,
    /// Pôle Nord détecté  la tension descend en dessous de VCC/2.
    NorthPole,
    /// Aucun champ magnétique significatif détecté (dans la zone morte).
    NoField,
}

// ---------------------------------------------------------------------------
// Implémentation
// ---------------------------------------------------------------------------

impl<'d> HallAnalog<'d> {
    /// Crée une nouvelle instance du driver `HallAnalog`.
    ///
    /// # Arguments
    ///
    /// * `adc`      Périphérique ADC Embassy en mode asynchrone.
    /// * `channel`  Canal ADC connecté à la broche AO du module Hall.
    ///
    /// # Exemple
    ///
    /// ```rust,no_run
    /// # use embassy_rp::adc::{Adc, Channel, Config as AdcConfig};
    /// # use embassy_hall_analog::HallAnalog;
    /// // let adc = Adc::new(p.ADC, Irqs, AdcConfig::default());
    /// // let channel = Channel::new_pin(p.PIN_26, embassy_rp::gpio::Pull::None);
    /// // let sensor = HallAnalog::new(adc, channel);
    /// ```
    #[inline]
    pub fn new(adc: Adc<'d, Async>, channel: Channel<'d>) -> Self {
        Self { adc, channel }
    }

    /// Lit la valeur brute du convertisseur ADC.
    ///
    /// | Cible   | Résolution | Plage        | Repos  |
    /// |---------|-----------|--------------|--------|
    /// | RP2040  | 12 bits   | `0..=4095`   | ~2048  |
    /// | RP235x  | 14 bits   | `0..=16383`  | ~8192  |
    ///
    /// En cas d'erreur ADC, la valeur `0` est retournée.
    ///
    /// # Exemple
    ///
    /// ```rust,no_run
    /// # use embassy_hall_analog::HallAnalog;
    /// # async fn example(mut sensor: HallAnalog<'_>) {
    /// let raw: u16 = sensor.read_raw().await;
    ///
    /// // Conversion en tension (V) RP2040 12 bits
    /// let voltage = raw as f32 * 3.3 / 4095.0;
    ///
    /// // Déviation par rapport au repos (ΔV)
    /// let delta_v = voltage - 1.65;
    ///
    /// // Estimation du champ en Gauss (sensibilité 49E @ 3.3V ≈ 0.92 mV/Gauss)
    /// let field_gauss = delta_v / 0.00092;
    /// # }
    /// ```
    #[inline]
    pub async fn read_raw(&mut self) -> u16 {
        self.adc.read(&mut self.channel).await.unwrap_or(0)
    }

    /// Lit la polarité du champ magnétique détecté.
    ///
    /// Le point de repos (`ZERO_FIELD_RAW`) est sélectionné automatiquement
    /// selon la feature compilée (`rp2040` → 2048, `rp235xa`/`rp235xb` → 8192).
    ///
    /// # Arguments
    ///
    /// * `deadband`  Demi-largeur de la zone morte en LSB.
    ///   Valeur typique : `50` sur 12 bits, `200` sur 14 bits.
    ///
    ///   | Condition                         | Résultat                          |
    ///   |-----------------------------------|-----------------------------------|
    ///   | `raw > ZERO + deadband`           | [`MagneticPolarity::SouthPole`]   |
    ///   | `raw < ZERO - deadband`           | [`MagneticPolarity::NorthPole`]   |
    ///   | sinon                             | [`MagneticPolarity::NoField`]     |
    ///
    /// # Exemple
    ///
    /// ```rust,no_run
    /// # use embassy_hall_analog::{HallAnalog, MagneticPolarity};
    /// # async fn example(mut sensor: HallAnalog<'_>) {
    /// match sensor.read_polarity(50).await {
    ///     MagneticPolarity::SouthPole => { /* pôle Sud */ }
    ///     MagneticPolarity::NorthPole => { /* pôle Nord */ }
    ///     MagneticPolarity::NoField   => { /* pas de champ */ }
    /// }
    /// # }
    /// ```
    pub async fn read_polarity(&mut self, deadband: u16) -> MagneticPolarity {
        let raw = self.read_raw().await;

        if raw > ZERO_FIELD_RAW.saturating_add(deadband) {
            MagneticPolarity::SouthPole
        } else if raw < ZERO_FIELD_RAW.saturating_sub(deadband) {
            MagneticPolarity::NorthPole
        } else {
            MagneticPolarity::NoField
        }
    }

    /// Lit la valeur brute et retourne la déviation signée par rapport au point de repos.
    ///
    /// Utile pour évaluer l'intensité relative du champ magnétique indépendamment
    /// de sa polarité, ou pour implémenter une logique de seuil personnalisée.
    ///
    /// # Arguments
    ///
    /// * `zero` Point de repos calibré en LSB.
    ///   Utilisez [`ZERO_FIELD_RAW_12BIT`] (RP2040) ou [`ZERO_FIELD_RAW_14BIT`] (RP235x),
    ///   ou une valeur mesurée sur votre module pour une meilleure précision.
    ///
    /// # Retour
    ///
    /// * `i32` — Déviation signée en LSB.
    ///   - Positif → pôle Sud
    ///   - Négatif → pôle Nord
    ///   - Proche de zéro → aucun champ
    ///
    /// # Exemple
    ///
    /// ```rust,no_run
    /// # use embassy_hall_analog::{HallAnalog, ZERO_FIELD_RAW_12BIT};
    /// # async fn example(mut sensor: HallAnalog<'_>) {
    /// let deviation = sensor.read_deviation(ZERO_FIELD_RAW_12BIT).await;
    /// // deviation > 0 : pôle Sud, deviation < 0 : pôle Nord
    /// # }
    /// ```
    pub async fn read_deviation(&mut self, zero: u16) -> i32 {
        let raw = self.read_raw().await;
        raw as i32 - zero as i32
    }
}