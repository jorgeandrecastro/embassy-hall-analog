[![crates.io](https://img.shields.io/crates/v/embassy-hall-analog.svg)](https://crates.io/crates/embassy-hall-analog)
[![docs.rs](https://docs.rs/embassy-hall-analog/badge.svg)](https://docs.rs/embassy-hall-analog)
[![License: GPL v2](https://img.shields.io/badge/License-GPL_v2-blue.svg)](https://www.gnu.org/licenses/old-licenses/gpl-2.0.en.html)

# embassy-hall-analog

Driver async `no_std` minimaliste pour le **capteur à effet Hall linéaire analogique OPEN-SMART**
(compatible 49E / SS49E) sur microcontrôleur **RP2040** et **RP235x**, basé sur le framework [Embassy](https://embassy.dev).

> ⚠️ Ce projet n'est pas développé par l'équipe officielle Embassy.

---

## ⚠️ Note importante sur le RP2350B (Pico 2 Zero / 2350B)

Si vous utilisez cette crate avec un RP2350B (modèle 80 pins), soyez vigilants sur le design de votre PCB. 

Sur certaines cartes de développement (comme la **Waveshare Pico Zero 2350B**), les broches ADC sont multiplexées ou partagées physiquement avec le bus de données de la carte SD (QSPI/SDIO). 
- **Symptôme :** Valeurs brutes (`read_raw`) instables, bruit important ou lecture "fantôme".
- **Solution :** Utilisez des broches ADC isolées du trafic numérique haute fréquence ou assurez-vous qu'aucun autre périphérique ne sollicite le bus partagé pendant la lecture analogique.

Le driver fonctionne parfaitement sur la **Pico 2 Standard** (RP2350A) sur les broches ADC dédiées (GP26 testé).

## 📄 Historique et Compatibilité

Ce projet suit de près l'évolution de l'écosystème Embassy pour garantir le support des nouvelles puces comme la RP2350.

**Dernière version stable conseillée : `0.3.0`** (ou supérieure).

**Important : Cette crate est compatible avec une large plage de versions (v0.4.0 à v0.10.0+).** Assurez-vous que votre projet utilise une version d'`embassy-rp` incluse dans cette plage.

Changelog : Pour voir le détail des changements, consultez le fichier `CHANGELOG.md`.

---

## Description

Le module OPEN-SMART repose sur un capteur à effet Hall linéaire de type **49E** qui produit une tension analogique proportionnelle à l'intensité du champ magnétique :

| Condition                  | Tension de sortie (@ 3,3 V) |
|----------------------------|-----------------------------|
| Aucun champ magnétique     | ~1,65 V (VCC / 2)           |
| Pôle Sud rapproché         | Augmente vers 3,3 V         |
| Pôle Nord rapproché        | Diminue vers 0 V            |

Ce driver encapsule la lecture ADC asynchrone Embassy et expose une API simple pour intégrer ce capteur dans vos projets embarqués RP2040 🟢 et RP2350 🟣.

---

## Câblage

Connexion directe — aucune résistance externe nécessaire (le module est auto-polarisé) :

```text
Module Hall OPEN-SMART        RP2040 / RP235x
──────────────────────        ───────────────
       VCC  ──────────────── 3.3V
       GND  ──────────────── GND
        AO  ──────────────── GP26 (ADC0)
```

> La broche AO (Analog Output) est connectée directement à la broche ADC du microcontrôleur.

---

## Installation

Ajoutez la dépendance dans votre `Cargo.toml`.

**Pour le RP2040 (Pico 1) — feature `rp2040` activée par défaut :**

```toml
[dependencies.embassy-hall-analog]
version = "0.3.1"
```

**Pour le RP2350A (Pico 2 A-step) — désactivez les features par défaut et activez `rp235xa` :**

```toml
[dependencies]
embassy-hall-analog = { version = "0.3.1", default-features = false, features = ["rp235xa"] }
```

**Pour le RP2350B (Waveshare RP2350-PiZero, Pico 2 B-step) — désactivez les features par défaut et activez `rp235xb` :**

```toml
[dependencies]
embassy-hall-analog = { version = "0.3.1", default-features = false, features = ["rp235xb"] }
```

> ⚠️ Ces trois features sont **mutuellement exclusives**. Le build échouera avec un message explicite
> si zéro ou plusieurs features cibles sont activées simultanément.

---

## Features

| Feature   | Description                        | Activée par défaut |
|-----------|------------------------------------|--------------------|
| `rp2040`  | Cible RP2040 (Raspberry Pi Pico 1) | ✓                  |
| `rp235xa` | Cible RP2350 A-step (Pico 2 A)     |                    |
| `rp235xb` | Cible RP2350 B-step (Pico 2 B)     |                    |

Le point de repos ADC (`ZERO_FIELD_RAW`) est sélectionné **automatiquement** selon la feature compilée :

| Feature             | Résolution | Point de repos théorique |
|---------------------|-----------|----------------|
| `rp2040`            | 12 bits   | 2048           |
| `rp235xa` / `rp235xb` | 14 bits | 8192           |

> ⚠️ **Important** : Les valeurs théoriques ne correspondent pas toujours à la réalité sur le matériel.
> Sur RP235x, bien que le matériel soit 14 bits, la valeur retournée peut être sur 12 bits selon la configuration du HAL Embassy.
> 
> **Valeur réelle mesurée (RP2350A) en l'absence de champ magnétique, branche sur 3.3V : ~2060**
>
> Cette variation dépend de la résistance interne du capteur, des variations thermiques et de la stabilité de l'alimentation.
>
> **→ Utilisez la méthode `calibrate()` lors du démarrage pour déterminer le point de repos réel de votre installation.**

---

## Utilisation

### Lecture brute

```rust
#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::adc::{Adc, Channel, Config as AdcConfig};
use embassy_rp::bind_interrupts;
use embassy_rp::adc::InterruptHandler;
use embassy_hall_analog::HallAnalog;

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => InterruptHandler;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let adc = Adc::new(p.ADC, Irqs, AdcConfig::default());
    let channel = Channel::new_pin(p.PIN_26, embassy_rp::gpio::Pull::None);

    let mut sensor = HallAnalog::new(adc, channel);

    loop {
        let raw = sensor.read_raw().await;
        // ~2048 (RP2040) ou ~8192 (RP235x) → pas de champ
        // > repos → pôle Sud  |  < repos → pôle Nord
        let _ = raw;
    }
}
```

### Détection de polarité

```rust
use embassy_hall_analog::MagneticPolarity;

match sensor.read_polarity(50).await {
    MagneticPolarity::SouthPole => { /* pôle Sud détecté */ }
    MagneticPolarity::NorthPole => { /* pôle Nord détecté */ }
    MagneticPolarity::NoField   => { /* pas de champ significatif */ }
}
```

> Le paramètre `deadband` (ici `50` LSB) définit la zone morte autour du point de repos
> pour éviter les oscillations dues au bruit ADC.  
> Sur RP235x (14 bits), une valeur de `200` LSB est conseillée.

### Calibration du point zéro

```rust
// Calibration avec 64 échantillons pour une grande précision
let zero_offset = sensor.calibrate(64).await;

// Ensuite, utiliser zero_offset pour les lectures précises
let deviation = sensor.read_deviation(zero_offset).await;
// deviation > 0 : pôle Sud | deviation < 0 : pôle Nord
```

> **Conseil** : Appelez `calibrate()` une seule fois au démarrage, dans un environnement sans champ magnétique perturbateur.
> Cela vous donnera le point de repos réel (~2060 au lieu de ~2048 sur RP235x par exemple).

### Déviation signée (approche manuelle)

```rust
use embassy_hall_analog::{HallAnalog, ZERO_FIELD_RAW_12BIT};

// Approche non recommandée (suppose un point de repos constant)
let deviation = sensor.read_deviation(ZERO_FIELD_RAW_12BIT).await;
// ⚠️ Cette valeur sera décalée si le point de repos réel ≠ 2048
```

### Conversions

**Valeur brute → tension (V) :**

```rust
// RP2040 (12 bits)
let voltage = (raw as f32 / 4095.0) * 3.3;

// RP235x (14 bits)
let voltage = (raw as f32 / 16383.0) * 3.3;
```

**Tension → champ magnétique estimé (Gauss) :**

```rust
// Sensibilité typique 49E @ 3,3 V ≈ 0,92 mV/Gauss
let delta_v = voltage - 1.65;          // déviation par rapport au repos
let field_gauss = delta_v / 0.00092;   // estimation en Gauss
```

---

## API

### `HallAnalog::new(adc, channel) -> Self`

Crée le driver en prenant possession de l'ADC Embassy et du canal correspondant.

### `async fn read_raw(&mut self) -> u16`

Lit la valeur ADC brute du capteur.

- RP2040 : 12 bits (`0..=4095`). Point de repos : ~2048.
- RP235x : 14 bits (`0..=16383`). Point de repos : ~8192.
- Retourne `0` en cas d'erreur ADC.

### `async fn read_polarity(&mut self, deadband: u16) -> MagneticPolarity`

Retourne la polarité détectée : `SouthPole`, `NorthPole`, ou `NoField`.

Le point de repos est sélectionné automatiquement selon la feature compilée  aucun paramètre `zero` à passer.

### `async fn read_deviation(&mut self, zero: u16) -> i32`

Retourne la déviation signée en LSB par rapport au point de repos fourni.

### `async fn calibrate(&mut self, samples: u8) -> u16`

Calibration automatique du point zéro.

Exécute une moyenne sur N échantillons pour déterminer le décalage (offset) réel du capteur
dans son environnement actuel. Utilise `embassy-time` pour insérer un délai de **100 μs**
entre chaque lecture, permettant à l'ADC de se stabiliser et d'éviter de lire le même pic de bruit.

**Exemple d'utilisation :**

```rust
// Calibration avec 64 échantillons pour une grande précision
let zero_offset = sensor.calibrate(64).await;
// Ensuite, utiliser zero_offset pour les lectures
let deviation = sensor.read_deviation(zero_offset).await;
```

> **Conseil :** Appelez `calibrate()` une seule fois au démarrage,
> dans un environnement sans champ magnétique perturbateur.

### Constantes

| Constante              | Valeur | Description                         |
|------------------------|--------|-------------------------------------|
| `ZERO_FIELD_RAW_12BIT` | 2048   | Point de repos ADC 12 bits (RP2040) |
| `ZERO_FIELD_RAW_14BIT` | 8192   | Point de repos ADC 14 bits (RP235x) |

---

## Compatibilité

| Dépendance     | Version     |
|----------------|-------------|
| `embassy-rp`   | 0.4 à 0.10+ |
| `embassy-time` | 0.3 à 0.6   |
| Rust edition   | 2024        |
| `no_std`       | ✓           |

---

## 📋 Changelog

Pour un historique détaillé des changements, consultez le fichier [CHANGELOG.md](CHANGELOG.md).

**Version actuelle : 0.3.0**
- ✅ Calibration automatique du point zéro
- ✅ Intégration de `embassy-time` pour la stabilisation ADC
- ✅ Amélioration de la précision des mesures

> **Important** : Les versions antérieures à 0.3.0 présentent des limitations critiques.
> Consultez le [CHANGELOG.md](CHANGELOG.md) pour les détails sur les versions héritées.

---


## Licence

Ce projet est distribué sous licence **GPL-2.0-or-later**.  
Voir le fichier [LICENSE](LICENSE) pour les détails complets.

---

## 🦅 À propos

Développé et testé par Jorge Andre Castro