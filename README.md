# Soulful

Soulful is a modern, self-hosted music downloader and manager. It bridges the gap between Soulseek (via `slskd`) and your music library (managed by `beets`), providing a seamless flow from search to streaming-ready library.

## Features

-   **Unified Search**: Search for albums and tracks using MusicBrainz metadata and find sources on Soulseek.
-   **One-Click Download & Import**: Select an album, choose your target folder, and Soulful handles the rest.
-   **Automated Importing**: Automatically monitors downloads and uses the `beets` CLI to tag, organize, and move files to your specified music folder.
-   **User Management**: Multi-user support with private folders. Each user can manage their own music library paths.
-   **Modern UI**: A responsive, clean interface built with Dioxus and Tailwind CSS.
-   **Private & Secure**: Self-hosted, login-protected (admin-managed registration).

## Architecture

1.  **Soulful Web**: The main interface (Dioxus Fullstack).
2.  **Slskd**: The Soulseek client backend. Soulful communicates with `slskd` to initiate and monitor downloads.
3.  **Beets**: The music library manager. Soulful executes `beet import` to process finished downloads.
4.  **SQLite**: Stores user accounts and folder configurations.

## Self-Hosting with Docker

The recommended way to run Soulful is via Docker Compose. This ensures all dependencies (like `beets` and `python`) are correctly set up.

### Prerequisites

-   Docker & Docker Compose
-   A running instance of [slskd](https://github.com/slskd/slskd)
-   Access to your music directories

### Quick Start

1.  Create a `docker-compose.yml` file:

```yaml
services:
  soulful:
    image: soulful:latest
    build: .
    restart: unless-stopped
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=sqlite:/data/soulful.db
      - SLSKD_URL=http://slskd:5030
      - SLSKD_API_KEY=your_slskd_api_key_here
      # The path where slskd saves files (INSIDE the soulful container)
      - SLSKD_DOWNLOAD_PATH=/downloads
      # Optional: Beets configuration
      - BEETS_CONFIG=/config/config.yaml
    volumes:
      # Data persistence (DB)
      - ./data:/data
      # Map the SAME download folder slskd uses
      - /path/to/slskd/downloads:/downloads
      # Map your music libraries (where beets will move files to)
      - /path/to/music:/music
    depends_on:
      - slskd

  # Example slskd service if you don't have one running
  slskd:
    image: slskd/slskd
    environment:
      - SLSKD_REMOTE_CONFIGURATION=true
    volumes:
      - ./slskd-config:/app/slskd.conf.d
      - /path/to/slskd/downloads:/app/downloads
    ports:
      - "5030:5030"
```

2.  **Important**: The `/downloads` volume must match between `slskd` and `soulful` so Soulful can see the files `slskd` downloaded.

3.  Build and Run:

```bash
docker-compose up -d --build
```

### Initial Setup

1.  Open `http://localhost:8080`
2.  Login with the default credentials:
    -   Username: `admin`
    -   Password: `admin`
3.  Go to **Settings**.
4.  **Change your password** (Create a new user if you prefer and delete the admin later, or just change the admin logic if you forked the code).
5.  **Add Music Folders**: Add the paths where you want your music to be stored (e.g., `/music/Rock`, `/music/Jazz`). These must be paths accessible inside the Docker container.

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | Connection string for SQLite | `sqlite:soulful.db` |
| `SLSKD_URL` | URL of your Slskd instance | `http://127.0.0.1:5030` |
| `SLSKD_API_KEY` | API Key for Slskd | *(Default dev key)* |
| `SLSKD_DOWNLOAD_PATH` | Path where Slskd downloads files | `/tmp/downloads` |
| `BEETS_CONFIG` | Path to custom beets config file | `beets_config.yaml` |

### Beets Configuration

Soulful uses `beets` to import music. You can mount a custom `config.yaml` to `/config/config.yaml` (or wherever you point `BEETS_CONFIG` to) to customize how beets behaves (plugins, naming formats, etc.).

Default `beet import` flags used:
-   `-q`: Quiet mode (no user interaction)
-   `-d [target_path]`: Import to the specific folder selected in the UI.

## Development

1.  Install Rust and `cargo-dx`.
2.  Run the tailwind watcher:
    ```bash
    ./css.sh
    ```
3.  Run the app:
    ```bash
    dx serve --platform web
