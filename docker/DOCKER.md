# Harper Docker Cookbook

This guide provides detailed instructions for running Harper using Docker.

## Prerequisites

- Docker installed and running
- Git (for cloning the repository)

## Quick Start

1. Clone the repository:
   ```bash
   git clone https://github.com/harpertoken/harper.git
   cd harper
   ```

2. Set up environment variables:
   ```bash
   cp env.example .env
   # Edit .env with your API keys (OpenAI, Sambanova, Gemini)
   ```

3. Build and run:
   ```bash
   docker build -t harper .
docker run --rm -it --env-file .env -v harper_data:/app/data harper
   ```

## Detailed Setup

### Building the Image

```bash
# Build with custom tag
docker build -t my-harper:latest .

# Build with no cache (force rebuild)
docker build --no-cache -t harper .
```

### Running the Container

#### Interactive Mode (Recommended)
```bash
# With environment variables
docker run --rm -it --env-file .env harper

# Without env vars (limited functionality)
docker run --rm -it harper
```

#### Background Mode
```bash
# Run in background
docker run -d --name harper-container --env-file .env harper

# View logs
docker logs harper-container

# Stop container
docker stop harper-container
```

### Using Docker Compose

Docker Compose provides persistent data storage and easier management.

```bash
# Start with compose
docker-compose up --build

# Run in background
docker-compose up -d --build

# View logs
docker-compose logs

# Stop
docker-compose down
```

### Data Persistence

Harper stores sessions in a SQLite database. With Docker Compose, data persists in a named volume.

To persist data with plain Docker:
```bash
# Mount host directory (Unix/Linux/macOS)
docker run --rm -it -v $(pwd)/data:/app/data --env-file .env harper

# Windows Command Prompt
docker run --rm -it -v %cd%/data:/app/data --env-file .env harper

# Windows PowerShell
docker run --rm -it -v ${pwd}/data:/app/data --env-file .env harper
```

### Troubleshooting

#### Common Issues

1. **GLIBC errors**: Ensure you're using a compatible base image (fixed in current Dockerfile).

2. **Config not found**: The Dockerfile includes config files; ensure the image is up-to-date.

3. **Menu looping**: Use `-it` flag for interactive mode.

4. **API key errors**: Verify your .env file has correct keys.

#### Debugging

```bash
# Check container logs
docker logs <container-id>

# Run with verbose output
docker run --rm -it --env-file .env -e RUST_LOG=debug harper

# Inspect running container
docker exec -it <container-id> /bin/bash
```

### Advanced Usage

#### Custom Configuration

Mount custom config:
```bash
docker run --rm -it -v $(pwd)/my-config:/app/config --env-file .env harper
```

#### Multi-Stage Builds

The Dockerfile uses multi-stage builds for optimized image size.

#### CI/CD

Docker builds are automatically tested in GitHub Actions on pushes to main/develop branches.

### Examples

#### Basic Chat Session
```bash
docker run --rm -it --env-file .env harper
# Select option 1 for new chat session
```

#### List Sessions
```bash
docker run --rm -it --env-file .env harper
# Select option 2 to list sessions
```

#### Export History
```bash
docker run --rm -it --env-file .env harper
# Select option 4 to export session history
```

## Support

For issues with Docker setup, check:
- Docker and Docker Compose versions
- Environment variables in .env
- Network connectivity for API calls
- Sufficient disk space for builds