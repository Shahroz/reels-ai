# AgentLoop

AgentLoop is a Rust (Actix Web) service designed to orchestrate iterative research sessions with Large Language Models (LLMs). It manages a conversation loop where an agent interacts with tools like web search and browsing based on user instructions, aiming to gather sufficient context to fulfill the research goal.

## Key Features
*   **No Database Required:** The core functionality operates without needing a persistent database connection.

*   **Interactive Research:** Accepts user instructions and initiates an LLM-driven research process.
*   **Tool Integration:** Utilizes tools (e.g., web search, website browsing) selected by the agent to gather information.
*   **Context Management:** Accumulates relevant information ("context") during the session.
*   **Background Evaluation:** A background process can monitor the context and signal when the research goal is likely met.
*   **Termination:** Sessions end when the goal is met, a time limit is reached, or manually terminated.
*   **WebSocket Streaming:** Provides real-time updates on agent reasoning, tool usage, and results via WebSockets.
*   **Embedded UI:** Includes a simple React-based chat interface served directly by the backend.

## Running with Docker

The easiest way to run AgentLoop is using Docker.

**Prerequisites:**
*   Docker installed.
*   A `.env` file in the `crates/agentloop` directory with necessary configurations (e.g., API keys, server address). See `.env.example` if available, or configure based on `crates/agentloop/src/config.rs`.

**Steps:**

1.  **Build the Docker Image:**
    Navigate to the `crates/agentloop` directory in your terminal. Use the provided Makefile target:
    ```bash
    make docker-build
    ```
    This command utilizes the `crates/agentloop/Dockerfile` to build the application, including the frontend UI, into a Docker image tagged `narrativ_app`.

2.  **Run the Docker Container:**
    Ensure your `.env` file is present in `crates/agentloop`. Use the Makefile target:
    ```bash
    make docker-run
    ```
    This command runs the container, mapping port 8080 (or as configured) on your host to the container's port and loading environment variables from the `.env` file.

3.  **Access the Application:**
    Once the container is running, you should be able to access the web UI, typically at `http://localhost:8080`. The API endpoints (like `/research` or `/ws`) will also be available on this address.

4.  **Stopping and Cleaning:**
    To stop the container:
    ```bash
    # Find the container ID
    docker ps
    # Stop the container
    docker stop <container_id>
    ```
    To clean up stopped containers based on the image:
    ```bash
    make docker-clean
    ```

## Development

Refer to the `Makefile` for other useful commands related to local development, database management (if applicable), testing, and API client generation.
