# Projet Rust - Client de Jeu Labyrinthe

## Introduction
Ce projet est un client Rust permettant de naviguer dans un labyrinthe et de résoudre des challenges en coopération avec d'autres joueurs. Il met en œuvre l'algorithme de Trémaux pour l'exploration du labyrinthe et gère le challenge `SecretSumModulo` en partageant des valeurs entre les joueurs.

## Installation et Configuration

### Prérequis
- **Rust & Cargo** (Si non installés, suivez les instructions sur [rust-lang.org](https://www.rust-lang.org/))
- **Un serveur accessible** pour exécuter le client.

### Installation
Clonez le dépôt et compilez le projet :
```sh
    git clone <URL_DU_REPO>
    cd <NOM_DU_REPO>
    cargo run --bin sauvequipeut
```

### Lancer le Client
Pour exécuter le client et rejoindre une partie :
```sh
    cargo run --bin sauvequipeut
```

## Utilisation
Le client tente de se connecter au serveur et suit plusieurs étapes :
1. Inscription de l'équipe
2. Connexion des joueurs
3. Navigation automatique dans le labyrinthe
4. Résolution du challenge `SecretSumModulo`
5. Recherche de la sortie (`G` sur le radar)

Les logs affichent les mouvements des joueurs et les interactions avec le serveur.

## Structure du Code

### **Fichiers Principaux**
- **`main.rs`** : Point d'entrée du programme.
- **`client.rs`** : Gère la connexion et le lancement des threads joueurs.
- **`game.rs`** : Contient la boucle principale du jeu et la gestion des interactions serveur.
- **`player.rs`** : Implémente les mécanismes de mouvement, de décision et de communication.
- **`utils.rs`** : Fonctions auxiliaires comme le décodage Base64.

### **Algorithmes Utilisés**
- **Exploration du Labyrinthe :**
    - Utilisation de l'algorithme de **Trémaux** pour l'exploration des chemins.
    - Mémorisation des cellules déjà visitées pour limiter les aller-retours inutiles.
    - Prise en compte des passages ouverts (extraction depuis `RadarView`).

- **Challenge `SecretSumModulo` :**
    - Agrège les secrets partagés entre les joueurs.
    - Applique une somme modulo fournie par le serveur.
    - Envoie la réponse formatée au serveur.

## Tests et Qualité du Code

### Lancer les tests unitaires
```sh
    cargo test
```
### Formater le code avec Rustfmt
```sh
    cargo fmt --all
```

### Documentation Rustdoc
Pour générer la documentation Rustdoc locale :
```sh
    cargo doc --open
```
Cela ouvrira la documentation dans le navigateur.

## Contribuer & Améliorations
- Si vous trouvez un bug ou souhaitez proposer une amélioration, ouvrez une **issue** sur le dépôt Git.
- Vous pouvez proposer des **pull requests** avec vos améliorations.

## Licence
Ce projet est sous licence MIT.

