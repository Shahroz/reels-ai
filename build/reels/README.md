# Project Build Process

This document explains how the application is built, both in the Google Cloud Build CI/CD pipeline and for local development. The process is designed to be secure and reproducible, with special handling for private Git dependencies.

The build is managed by two key files:
* `cloudbuild.yaml`: The CI/CD pipeline definition for Google Cloud Build.
* `build/narrativ/Dockerfile`: The instructions for building the final application image.

---
## High-Level Strategy

The core strategy is to use a self-contained `Dockerfile` that can build the entire application from source. The CI/CD pipeline's main responsibility is to securely inject the necessary credentials (a GitHub Personal Access Token) into the Docker build process at runtime.

This is achieved using **Docker BuildKit**, which allows us to securely mount secret files into `RUN` commands without ever saving them to the final image layers.

---
## Component Breakdown

### ## `cloudbuild.yaml`
This file orchestrates the build in the CI environment. It performs two main steps:

1.  **Create a Git Config File:** It uses a secret GitHub PAT from Google Secret Manager to create a temporary `.gitconfig` file. This file contains a `url.insteadOf` rule that tells Git to automatically rewrite any `https://github.com` URL to include the token for authentication.

2.  **Run `docker build`:** It executes the `docker build` command, passing the temporary `.gitconfig` file into the build process using the `--secret=id=gitconfig,...` flag. This makes the credentials available to the `Dockerfile` without exposing them directly.

### ## `Dockerfile`
This is a multi-stage `Dockerfile` that handles building the frontend, fetching dependencies, and compiling the Rust backend into a minimal final image.

It solves the private dependency problem with two key features:

1.  **`COPY .cargo/config.toml`**: The repository contains a small configuration file that tells Rust's build tool, Cargo, to use the system `git` command for fetching dependencies. This is essential for ensuring it respects the `.gitconfig` file we provide.

2.  **`RUN --mount=type=secret,...`**: Any command that needs to access the private repository (like `cargo fetch` or `cargo build`) uses the `--mount` flag. This command temporarily makes the secret `.gitconfig` file (provided by Cloud Build) available. **This is highly secure**, as the file is never written to the disk of the final image.

---
## Local Development

You can build the Docker image locally by replicating the CI process. This requires Docker with BuildKit enabled (standard on modern Docker Desktop).

### ### 1. Create a Local Git Credentials File
Create a file named `.gitconfig.local` in the root of the repository. **Do not commit this file.**

**`.gitconfig.local`:**
```ini
[url "https://x-access-token:YOUR_GITHUB_PAT_HERE@github.com/"]
    insteadOf = [https://github.com/](https://github.com/)
```

* Replace YOUR_GITHUB_PAT_HERE with your own GitHub Personal Access Token that has repo scope.

* Add .gitconfig.local to your main .gitignore file to prevent it from ever being committed.

### 2. Run the Docker Build Command
Execute the docker build command from the repository root, pointing it to your local secret file:


```bash
docker build \
  --secret=id=gitconfig,src=.gitconfig.local \
  -f build/narrativ/Dockerfile \
  -t my-narrativ-service .
This command tells Docker to use your local credential file for the gitconfig secret, just like Cloud Build does in the pipeline.
```

Cors
####

If cors error happens this has to be

gsutil cors set cors.json gs://bounti_prod_narrativ_public