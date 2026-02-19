# Bilbo

Bibliothèque numérique publique en ligne. Bilbo indexe des livres numérisés (OCR) importés sous forme de fichiers markdown avec frontmatter YAML. Le site propose recherche par mots-clefs (PostgreSQL FTS), recherche contextuelle (Qdrant + Mistral embeddings), et un chat Q&A (RAG).

## Stack technique

| Couche | Technologie |
|---|---|
| Frontend | [Leptos](https://leptos.dev/) 0.8 (Rust/WASM) avec SSR |
| Backend | [Axum](https://github.com/tokio-rs/axum) 0.8 |
| Base relationnelle | PostgreSQL 17 (Supabase Cloud en prod, Docker en dev) |
| Base vectorielle | [Qdrant](https://qdrant.tech/) v1.13 (Qdrant Cloud en prod, Docker en dev) |
| LLM | [Mistral AI](https://mistral.ai/) (embeddings `mistral-embed`, chat `mistral-small-latest`) |
| Toolchain | Rust nightly + `wasm32-unknown-unknown` via [Nix flakes](https://nixos.org/) |
| Build | [cargo-leptos](https://github.com/leptos-rs/cargo-leptos) |
| Migrations | [sqlx-cli](https://github.com/launchbadge/sqlx) |
| Styles | SCSS (compilé par cargo-leptos via dart-sass) |

## Prérequis

- [Nix](https://nixos.org/download/) avec flakes activés
- [direnv](https://direnv.net/) (recommandé)
- [Docker](https://docs.docker.com/get-docker/) et Docker Compose
- Une clé API [Mistral AI](https://console.mistral.ai/)

## Démarrage rapide

### 1. Cloner et entrer dans le shell de dev

```sh
git clone <repo-url> bilbo && cd bilbo

# Avec direnv (automatique) :
direnv allow

# Ou manuellement :
nix develop
```

Le shell Nix fournit : Rust nightly, cargo-leptos, sqlx-cli, dart-sass, binaryen.

### 2. Configurer l'environnement

```sh
cp .env.example .env
```

Editer `.env` :

```
DATABASE_URL=postgres://bilbo:bilbo_dev@localhost:5432/bilbo
QDRANT_URL=http://localhost:6334
MISTRAL_API_KEY=sk-xxxxxxxxxxxxxxxxxxxxx
```

### 3. Lancer les services Docker

```sh
docker compose up -d
```

Cela démarre :
- **PostgreSQL** sur `localhost:5432`
- **Qdrant** sur `localhost:6333` (REST) et `localhost:6334` (gRPC)

### 4. Appliquer les migrations

Les migrations sont appliquées automatiquement au démarrage du serveur. Pour les lancer manuellement :

```sh
sqlx migrate run
```

### 5. Lancer le serveur de dev

```sh
cargo leptos watch
```

Le site est disponible sur **http://localhost:3000**. Le rechargement à chaud est actif (port 3001).

## Import de livres

### Format des fichiers

Déposer des fichiers `.md` dans le dossier `data/`. Chaque fichier doit contenir un frontmatter YAML :

```markdown
---
id: MON-LIVRE-001
title: "Le Titre du Livre"
authors:
  - "Prénom Nom"
editor: "Nom de l'éditeur"
tags:
  - philosophie
  - essai
edition_date: "2023"
summary: "Résumé optionnel du livre."
introduction: "Texte d'introduction optionnel."
cover_text: "Texte de la 4e de couverture."
ean: "9781234567890"
isbn: "978-1-23-456789-0"
reseller_paper_urls:
  - "https://www.libraire.fr/mon-livre"
reseller_digital_urls:
  - "https://www.numerique.fr/mon-livre"
---

# Chapitre 1 - Introduction

Contenu OCR du livre ici...

# Chapitre 2 - Développement

Suite du contenu...
```

Champs obligatoires : `id`, `title`. Tous les autres sont optionnels. Si `summary` est absent et qu'une clé Mistral est configurée, un résumé sera généré automatiquement.

### Lancer l'import

```sh
cargo leptos serve -- --import
```

Le pipeline d'import :
1. Parse le frontmatter YAML et le contenu markdown
2. Calcule un hash SHA-256 (skip si inchangé)
3. Génère résumé et résumés de chapitres via Mistral (si absents)
4. Insère/met à jour les métadonnées dans PostgreSQL
5. Découpe le contenu en chunks (~2000 caractères, 400 de chevauchement)
6. Génère les embeddings via `mistral-embed` (1024 dimensions)
7. Indexe les vecteurs dans Qdrant
8. Déplace le fichier dans `data/processed/` (ou `data/failed/` en cas d'erreur)

## Structure du projet

```
bilbo/
├── flake.nix                     # Nix flake (toolchain Rust, cargo-leptos, sqlx-cli)
├── docker-compose.yml            # PostgreSQL + Qdrant
├── Cargo.toml                    # Crate unique, features ssr/hydrate
├── migrations/
│   └── 20260219000001_init.sql   # Schéma PostgreSQL
├── public/
│   └── robots.txt
├── style/
│   └── main.scss                 # Styles SCSS (responsive)
├── data/                         # Fichiers markdown à importer
└── src/
    ├── main.rs                   # Point d'entrée serveur (CLI --import)
    ├── lib.rs                    # Point d'entrée WASM (hydration)
    ├── app.rs                    # Composant racine, routeur Leptos
    ├── api.rs                    # Server functions (#[server])
    ├── error.rs                  # Types d'erreur
    ├── model/
    │   └── book.rs               # Types partagés (Book, ChatMessage, etc.)
    ├── pages/
    │   ├── home.rs               # Recherche et listing
    │   ├── book_detail.rs        # Détail livre (SEO, JSON-LD)
    │   └── chat.rs               # Chat Q&A (RAG)
    ├── components/
    │   ├── layout.rs             # Header, footer, navigation
    │   ├── search_bar.rs         # Barre de recherche (keyword/sémantique)
    │   ├── book_table.rs         # Tableau paginé de livres
    │   ├── chat_widget.rs        # Interface de chat
    │   ├── reseller_links.rs     # Liens vers les revendeurs
    │   └── tag_badge.rs          # Badge tag cliquable
    └── server/                   # SSR uniquement
        ├── state.rs              # AppState (PgPool, Qdrant, Mistral)
        ├── db.rs                 # Requêtes PostgreSQL
        ├── qdrant.rs             # Client Qdrant (collection, upsert, search)
        ├── mistral.rs            # Client Mistral AI (embeddings, chat)
        ├── import.rs             # Pipeline d'import
        ├── markdown.rs           # Parsing frontmatter + chunking
        └── api.rs                # Handler sitemap.xml
```

## Pages et SEO

| Route | Description |
|---|---|
| `/` | Recherche (mots-clefs ou sémantique), filtres par tag/auteur, tableau paginé |
| `/book/:ref_id` | Détail du livre avec meta tags, Open Graph, JSON-LD Schema.org `Book` |
| `/chat` | Chat conversationnel Q&A avec citations de sources |
| `/sitemap.xml` | Sitemap XML généré depuis la base de données |

Le SSR assure que les crawlers voient le contenu complet dès le premier rendu.

## Variables d'environnement

| Variable | Description | Obligatoire |
|---|---|---|
| `DATABASE_URL` | URL de connexion PostgreSQL | Oui |
| `QDRANT_URL` | URL gRPC de Qdrant (défaut : `http://localhost:6334`) | Non |
| `MISTRAL_API_KEY` | Clé API Mistral AI | Non (désactive embeddings/chat/résumés auto) |

## Commandes utiles

```sh
# Dev avec rechargement à chaud
cargo leptos watch

# Build de production
cargo leptos build --release

# Lancer l'import
cargo leptos serve -- --import

# Lancer les migrations manuellement
sqlx migrate run

# Arrêter les services Docker
docker compose down

# Supprimer les volumes Docker (reset complet)
docker compose down -v
```
