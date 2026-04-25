[![crates.io](https://img.shields.io/crates/v/embassy-hall-analog.svg)](https://crates.io/crates/embassy-hall-analog)
[![docs.rs](https://docs.rs/embassy-hall-analog/badge.svg)](https://docs.rs/embassy-hall-analog)
[![License: GPL v2](https://img.shields.io/badge/License-GPL_v2-blue.svg)](https://www.gnu.org/licenses/old-licenses/gpl-2.0.en.html)

# embassy-hall-analog

Driver async `no_std` minimaliste pour le **capteur à effet Hall linéaire analogique OPEN-SMART**
(compatible 49E / SS49E) sur microcontrôleur **RP2040** et **RP235x**, basé sur le framework [Embassy](https://embassy.dev) , ce n'est pas un projet développé par embassy officiel attention!!!.

---

## 📄 Historique et Compatibilité

Ce projet suit de près l'évolution de l'écosystème Embassy pour garantir le support des nouvelles puces comme la RP2350.

**Dernière version stable conseillée : `0.1.0`** (ou supérieure).

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

```
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

**Pour le RP2040 (Pico 1)  feature `rp2040` activée par défaut :**

```toml
[dependencies.embassy-hall-analog]
version = "0.1.0"
```

**Pour le RP235x (Pico 2) désactivez les features par défaut et activez `rp235x` :**

```toml
[dependencies]
embassy-hall-analog = { version = "0.1.0", default-features = false, features = ["rp235x"] }
```

**Important : Cette crate est compatible avec une large plage de versions (v0.4.0 à v0.10.0+).** Assurez-vous que votre projet utilise une version d'`embassy-rp` incluse dans cette plage.

---

## Features

| Feature    | Description                          | Activée par défaut |
|------------|--------------------------------------|--------------------|
| `rp2040`   | Cible RP2040 (Raspberry Pi Pico 1)   | ✓                  |
| `rp235x`   | Cible RP235x (Raspberry Pi Pico 2)   |                    |

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

### Déviation signée

```rust
use embassy_hall_analog::{HallAnalog, ZERO_FIELD_RAW_12BIT};

let deviation = sensor.read_deviation(ZERO_FIELD_RAW_12BIT).await;
// deviation > 0 : pôle Sud | deviation < 0 : pôle Nord
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

### `async fn read_deviation(&mut self, zero: u16) -> i32`

Retourne la déviation signée en LSB par rapport au point de repos fourni.

### Constantes

| Constante               | Valeur | Description                           |
|-------------------------|--------|---------------------------------------|
| `ZERO_FIELD_RAW_12BIT`  | 2048   | Point de repos ADC 12 bits (RP2040)   |
| `ZERO_FIELD_RAW_14BIT`  | 8192   | Point de repos ADC 14 bits (RP235x)   |

---

## Compatibilité

| Dépendance   | Version     |
|--------------|-------------|
| `embassy-rp` | 0.4 à 0.10+ |
| Rust edition | 2024        |
| `no_std`     | ✓           |

---

## Exemple complet : Pico 2040 avec affichage OLED

Utilise [`embassy-ssd1306`](https://crates.io/crates/embassy-ssd1306).

```rust
#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::adc::{Adc, Config as AdcConfig, Channel, InterruptHandler as AdcInterruptHandler};
use embassy_rp::i2c::{Config as I2cConfig, I2c, Async};
use embassy_time::{Timer, Duration};
use embassy_rp::gpio::{Output, Level, Pull};
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::I2C0;
use embassy_rp::i2c::InterruptHandler as I2cInterruptHandler;
use embassy_ssd1306::Ssd1306;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_hall_analog::{HallAnalog, MagneticPolarity, ZERO_FIELD_RAW_12BIT};
use {panic_halt as _, embassy_rp as _};

bind_interrupts!(struct Irqs {
    I2C0_IRQ      => I2cInterruptHandler<I2C0>;
    ADC_IRQ_FIFO  => AdcInterruptHandler;
});

#[embassy_executor::task]
async fn system_task(
    mut oled: Ssd1306<I2cDevice<'static, NoopRawMutex, I2c<'static, I2C0, Async>>>,
    mut hall: HallAnalog<'static>,
) {
    if let Ok(_) = oled.init().await {
        oled.clear();
        let _ = oled.flush().await;
    }

    loop {
        oled.clear();
        oled.draw_rect(0, 0, 127, 63, true);

        let raw = hall.read_raw().await;
        let deviation = hall.read_deviation(ZERO_FIELD_RAW_12BIT).await;

        oled.draw_str(10, 1, b"Hall Sensor");
        oled.draw_str(10, 3, b"Raw :");
        oled.draw_i16(55, 3, raw as i16);
        oled.draw_str(10, 5, b"Dev :");
        oled.draw_i16(55, 5, deviation as i16);

        let polarity = hall.read_polarity(50).await;
        oled.draw_str(10, 7, b"Field:");
        match polarity {
            MagneticPolarity::SouthPole => oled.draw_str(65, 7, b"Sud  "),
            MagneticPolarity::NorthPole => oled.draw_str(65, 7, b"Nord "),
            MagneticPolarity::NoField   => oled.draw_str(65, 7, b"None "),
        }

        let _ = oled.flush().await;
        Timer::after(Duration::from_millis(500)).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // I2C & OLED
    let mut i2c_config = I2cConfig::default();
    i2c_config.frequency = 400_000;
    let i2c_bus = I2c::new_async(p.I2C0, p.PIN_5, p.PIN_4, Irqs, i2c_config);

    static I2C_BUS: static_cell::StaticCell<Mutex<NoopRawMutex, I2c<'static, I2C0, Async>>>
        = static_cell::StaticCell::new();
    let i2c_mutex = I2C_BUS.init(Mutex::new(i2c_bus));

    let i2c_dev_oled = I2cDevice::new(i2c_mutex);
    let oled = Ssd1306::new(i2c_dev_oled, 0x3C);
    let mut led = Output::new(p.PIN_25, Level::Low);

    // ADC & capteur Hall
    let adc = Adc::new(p.ADC, Irqs, AdcConfig::default());
    let pin26 = Channel::new_pin(p.PIN_26, Pull::None);
    let hall = HallAnalog::new(adc, pin26);

    spawner.spawn(system_task(oled, hall)).unwrap();

    // Blink LED Pico pin 25
    loop {
        led.toggle();
        Timer::after_millis(200).await;
    }
}
```

---

## Licence

Ce projet est distribué sous licence **GPL-2.0-or-later**.  
Voir le fichier [LICENSE](LICENSE) pour les détails complets.

---

## 🦅 À propos

Développé et testé par Jorge Andre Castro