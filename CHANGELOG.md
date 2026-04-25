# Changelog

Tous les changements notables de ce projet sont documentés dans ce fichier.

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

**Les versions 0.3.0+ sont fortement recommandées pour tout nouveau projet.**

---

## Format de ce changelog

Ce projet respecte [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

Les versions suivent [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
