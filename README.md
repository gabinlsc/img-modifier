# img-modifier

Un CLI ultra-rapide et multithread écrit en **Rust** pour le recadrage, le filtrage et la compilation en lot d'images vers un PDF unique.

---

## Fonctionnalités

- **Recadrage carré automatique (1:1)** centré
- **Filtres intégrés :** Noir et blanc (`bw`), négatif (`invert`) et réglage de la luminosité
- **Multithreading natif :** Traitement en parallèle sur tous les cœurs du CPU via **Rayon**
- **Progression temps réel :** Barre d'avancement CLI avec temps restant via **Indicatif**
- **Scannage de dossiers :** Prise en charge de fichiers isolés ou de répertoires complets
- **Compilation PDF :** Regroupe les images retouchées dans un PDF au format A4

---

## Installation & compilation

Prérequis : Rust et Cargo installés.

```bash
git clone https://github.com/gabinlsc/img-modifier.git
cd img-modifier
cargo build --release
```

Le binaire optimisé se trouve dans `target/release/img-modifier`.

---

## 🛠️ Exemples d'utilisation

### 1. Recadrer en carré et passer en noir et blanc
```bash
cargo run --release -- ./photos/image.jpg --square --filter bw --output-dir ./resultats
```

### 2. Traiter tout un dossier en parallèle et augmenter la luminosité
```bash
cargo run --release -- ./mon_dossier/ --brightness 20 --output-dir ./export
```

### 3. Retoucher et compiler l'ensemble en un PDF
```bash
cargo run --release -- ./documents/ --square --filter bw --to-pdf rapport.pdf
```

---

## Options du CLI

| Option | Alias | Description |
|---|---|---|
| `<INPUTS>...` | | Fichiers ou dossiers d'entrée |
| `--filter` | `-f` | Filtre à appliquer (`bw`, `invert`) |
| `--square` | `-s` | Recadre au format carré 1:1 centré |
| `--brightness` | | Ajuste la luminosité (ex: `-20`, `30`) |
| `--output-dir` | `-o` | Dossier de sortie (défaut : `./output`) |
| `--to-pdf` | | Exporte la sélection dans un fichier PDF |

---

## Stack technique

- **clap** (v4) — Parsing et validation des arguments en ligne de commande
- **image** — Décodage, encodage et manipulation matricielle des pixels
- **rayon** — Parallélisation native des calculs par lot
- **indicatif** — Rendu de barres de progression dynamiques dans le terminal
- **printpdf** — Génération vectorielle du document PDF