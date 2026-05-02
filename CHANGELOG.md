# Changelog

Tous les changements notables de ce projet sont documentés dans ce fichier.



## [0.4.0] - 2026-05-02

- Remplacement des plages de compatibilité par des versions explicites afin d’assurer une meilleure stabilité et reproductibilité des builds.

### Dépendances

```toml
[dependencies]

embassy-rp = "0.10"
embassy-time = "0.5"

```


## [0.3.0] - 2026-04-25

### ✨ Ajouts

- **Méthode `calibrate()`** : Calibration automatique du point zéro avec moyenne d'échantillons
- **Intégration de `embassy-time`** : Ajout de délais (100 µs) entre les lectures de calibration pour stabiliser l'ADC

### 🚀 Améliorations

- **Précision accrue** : La calibration évite maintenant de lire des pics de bruit successifs
- **Documentation complète** : Exemples d'utilisation pour la calibration et la détection de polarité
- **Tableaux de compatibilité** : Alignement avec les versions d'embassy-time

### ✅ Rétrocompatibilité

- Toutes les API précédentes restent inchangées
- Migration facile depuis les versions 0.2.x

### 📌 Avertissements et limitations matérielles

#### RP2350B (Pico 2 Zero / 80 pins)

**⚠️ Important** : Sur certaines cartes de développement (notamment la **Waveshare Pico Zero 2350B**), les broches ADC sont **multiplexées ou partagées physiquement** avec le bus de données de la carte SD (QSPI/SDIO).

- **Symptôme** : Valeurs brutes (`read_raw`) instables, bruit important ou lectures "fantômes"
- **Cause racine** : Interférences dues au trafic numérique haute fréquence sur le bus partagé
- **Solution** : 
  - Utilisez des broches ADC isolées du trafic numérique haute fréquence
  - Assurez-vous qu'aucun autre périphérique ne sollicite le bus partagé pendant les lectures analogiques
  - Envisagez d'ajouter du filtrage analogique (condensateur 100 nF en parallèle)

#### RP2350A (Pico 2 Standard)

✅ **Fonctionne parfaitement** sur les broches ADC dédiées (GP26 testé).

#### Calibration matérielle et points de repos réels

**Important pour la précision** : Les points de repos théoriques (~2048 sur 12 bits, ~8192 sur 14 bits) 
ne correspondent **pas toujours** à la réalité sur le matériel.

**Valeurs réelles mesurées en absence de champ magnétique (branche sur 3.3V)** :
- RP2350A avec calibration : **~2060** (au lieu de 2048)
- Variation due à : résistance interne du capteur, thermique, stabilité de l'alimentation

**Recommandation** : Appelez toujours `calibrate()` au démarrage pour obtenir le point de repos réel de votre installation.

---

## ⚠️ Versions antérieures (< 0.3.0)

### Limitations critiques de v0.2.0 et antérieures

Les versions 0.2.0 et antérieures présentent des défauts fondamentaux rendant impossibles les mesures précises :

#### 🔴 Problèmes identifiés

- **Pas de calibration automatique** : Impossibilité d'adapter le point de repos à l'environnement réel
- **Pas de stabilisation ADC** : Lectures de calibration successives sans délai, capturant le même pic de bruit
- **Conception trop idéaliste** : Supposaient que l'ADC commencerait toujours à 50% de VCC
  - RP2040 (12 bits) : Attendait 2048 constant
  - RP235x (14 bits) : Attendait 8192 constant
- **Précision insuffisante** : Les mesures dérivent fortement avec la température et les variations de composants
- **Absence de contrôle de l'offset** : Aucun mécanisme pour corriger les décalages réels du capteur

#### 🚨 Conséquences pratiques

- Détections de polarité imprécises
- Déviation signée incorrecte sur longue durée
- Comportement imprévisible lors de changements thermiques
- Incompatibilité avec les modules Hall réels (qui varient naturellement)

#### ✅ Migration recommandée

```rust
// ❌ AVANT (0.2.0) - Non recommandé
let deviation = sensor.read_deviation(ZERO_FIELD_RAW_12BIT).await;

// ✅ APRÈS (0.3.0+) - Recommandé
let zero_offset = sensor.calibrate(64).await;  // Une seule fois au démarrage
let deviation = sensor.read_deviation(zero_offset).await;  // Lectures précises
```

**Les versions 0.4 + sont fortement recommandées pour tout nouveau projet.**

---

## Format de ce changelog

Ce projet respecte [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

Les versions suivent [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
