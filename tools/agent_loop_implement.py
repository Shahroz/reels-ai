import asyncio
from collections import defaultdict, deque
import os
import shlex # Use shlex for safer command construction (though not strictly necessary here as we control inputs)

# --- Configuration ---

# Ensure the script can be run from the project root or its directory
# Assumes the script is in the project root alongside 'crates' and 'rust_guidelines'
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = SCRIPT_DIR # Adjust if the script is placed elsewhere

# Define base command - adjust path to 'zen' executable if needed
# Assuming zen is in PATH or adjust accordingly
ZEN_CMD = "zen do"

# Define standard file paths often needed as patterns
RUST_GUIDELINES_FILE = os.path.join(PROJECT_ROOT, "rust_coding_guidelines.md")
BRIEF_V2_FILE = os.path.join(PROJECT_ROOT, "crates/agentloop/brief_v2.md")
DATA_STRUCTURES_FILE = os.path.join(PROJECT_ROOT, "crates/agentloop/data_structures.md")
FUNCTION_SIGNATURES_FILE = os.path.join(PROJECT_ROOT, "crates/agentloop/function_signatures.md")

# --- Task Definition Class ---

class Task:
    def __init__(self, id, description, command_instruction, patterns=None):
        """
        Initializes a Task. All file paths and globs are passed via --patterns.

        Args:
            id (str): Unique identifier for the task.
            description (str): Human-readable description.
            command_instruction (str): The instruction for the 'zen do' command.
            patterns (list[str], optional): List of absolute paths and glob patterns
                                            for file matching (passed to zen).
                                            This includes both files to be potentially
                                            modified and files providing context.
                                            Defaults to [].
        """
        self.id = id
        self.description = description
        self.patterns = patterns or []

        # Construct patterns arguments string
        # Ensure uniqueness to avoid redundant --patterns arguments
        unique_patterns = sorted(list(set(self.patterns))) + ["**"] + ["**/*.*"]
        pattern_args_str = "--patterns " + ",".join(unique_patterns)

        # Construct the full command using only --patterns
        command_instruction = command_instruction.replace('"',' ').replace("`"," ") + " IMPLEMENT IT FULLY - you should have all the information"
        self.command = f'{ZEN_CMD} --instruction "{command_instruction}" {pattern_args_str}'.strip()

        # Dependency tracking
        self.dependencies = set()
        self.dependents = set()

    def __repr__(self):
        return f"Task(id='{self.id}')"

# --- Task Definitions ---

tasks = {}

# Helper function to create absolute paths for patterns relative to project root
def rp(*paths):
    """Creates an absolute path relative to the project root."""
    return os.path.join(PROJECT_ROOT, *paths)

# 1. Directory Structure Setup
tasks["create_dirs"] = Task(
    "create_dirs",
    "Create base source directories for the agentloop crate",
    "Create directories `crates/agentloop/src/types`, `crates/agentloop/src/models`, `crates/agentloop/src/config`, `crates/agentloop/src/state`, `crates/agentloop/src/auth`, `crates/agentloop/src/handlers`, `crates/agentloop/src/session`, `crates/agentloop/src/tools`, `crates/agentloop/src/conversation`, `crates/agentloop/src/background`, `crates/agentloop/src/websocket`, `crates/agentloop/src/utils`, and `crates/agentloop/ui` if they don't exist.",
    patterns=[rp("crates/agentloop/src/")] # Pattern indicates the area of interest/modification
)

# 2. Common Types & Dependencies
tasks["define_common_types"] = Task(
    "define_common_types",
    "Define common type aliases SessionId and Timestamp",
    "In `crates/agentloop/src/types/mod.rs` (create if needed), define `pub type SessionId = uuid::Uuid;` and `pub type Timestamp = chrono::DateTime<chrono::Utc>;`. Ensure `uuid` and `chrono` crates are added to `crates/agentloop/Cargo.toml` with features `uuid = { version = \"0.8\", features = [\"v4\", \"serde\"] }` and `chrono = { version = \"0.4\", features = [\"serde\"] }`. Adhere to rust_guidelines.",
    patterns=[
        RUST_GUIDELINES_FILE,
        rp("crates/agentloop/src/types/mod.rs"),
        rp("crates/agentloop/Cargo.toml")
    ]
)

# 3. Data Structures (Based on data_structures.md)
data_structure_files = [
    # filename_stem, StructName, target_dir_subpath
    ("session_status", "SessionStatus", "types"),
    ("research_request", "ResearchRequest", "types"),
    ("research_response", "ResearchResponse", "types"),
    ("termination_request", "TerminationRequest", "types"),
    ("status_response", "StatusResponse", "types"),
    ("session_config", "SessionConfig", "types"),
    ("compaction_policy", "CompactionPolicy", "types"),
    ("sender", "Sender", "types"),
    ("conversation_entry", "ConversationEntry", "types"),
    ("context_entry", "ContextEntry", "types"),
    ("tool_choice", "ToolChoice", "types"),
    ("tool_result", "ToolResult", "types"),
    ("context_evaluator_feedback", "ContextEvaluatorFeedback", "types"),
    ("agent_error", "AgentError", "types"),
    ("ws_request", "WebsocketRequest", "types"),
    ("ws_event", "WebsocketEvent", "types"),
    ("session_data", "SessionData", "models"), # Note: Placed in models as per original structure idea
    ("app_state", "AppState", "state"), # Note: Placed in state as it relates to overall app state
]

# Base patterns needed for most struct definitions
struct_patterns_base = [
    DATA_STRUCTURES_FILE,
    RUST_GUIDELINES_FILE,
    rp("crates/agentloop/src/types/mod.rs"),
    rp("crates/agentloop/src/types/**/*.rs"), # Include existing types as context
    rp("crates/agentloop/src/models/**/*.rs"), # Include existing models as context
]

for file_stem, struct_name, dir_type in data_structure_files:
    task_id = f"define_{file_stem}"
    file_path = rp(f"crates/agentloop/src/{dir_type}/{file_stem}.rs")
    mod_path = rp(f"crates/agentloop/src/{dir_type}/mod.rs")
    instruction = (
        f"Create the file `{file_path}` containing the Rust struct `{struct_name}` as defined in `crates/agentloop/data_structures.md`. "
        f"Ensure it derives Debug, Clone, Serialize, Deserialize and includes necessary imports (like SessionId, SessionStatus etc. using fully qualified paths, e.g., `crate::types::SessionId`). "
        f"Add file-level documentation following Rust conventions. Add `pub mod {file_stem};` to `{mod_path}` (create mod.rs if needed). "
        f"Adhere strictly to `rust_guidelines` (one primary item per file, fully qualified paths, preamble docs)."
    )
    tasks[task_id] = Task(
        task_id,
        f"Define the {struct_name} data structure and update its parent mod.rs",
        instruction,
        # Combine base patterns with the specific files being created/modified
        patterns=struct_patterns_base + [file_path, mod_path]
    )

# 4. Function Signatures & Core Logic (Based on function_signatures.md)

# Base patterns needed for many function implementations
impl_patterns_base = [
    FUNCTION_SIGNATURES_FILE,
    RUST_GUIDELINES_FILE,
    rp("crates/agentloop/src/types/**/*.rs"),
    rp("crates/agentloop/src/models/**/*.rs"),
    rp("crates/agentloop/src/state/app_state.rs"), # AppState is commonly used
]

tasks["implement_app_bootstrap"] = Task(
    "implement_app_bootstrap",
    "Implement Application Bootstrap functions",
    "Implement `load_env_config`, `init_app_state`, `configure_app` based on signatures in `function_signatures.md`. Place `load_env_config` in `crates/agentloop/src/config/mod.rs`, `init_app_state` in `crates/agentloop/src/state/mod.rs`, and `configure_app` likely in `crates/agentloop/src/main.rs` or `crates/agentloop/src/app_setup.rs` (create if needed). Use the defined `AppConfig` (define simply if needed) and `AppState` types. Adhere to `rust_guidelines`.",
    patterns=impl_patterns_base + [
        rp("crates/agentloop/src/config/mod.rs"),
        rp("crates/agentloop/src/state/mod.rs"),
        rp("crates/agentloop/src/main.rs"),
        rp("crates/agentloop/src/app_setup.rs"), # Include potential new file
    ]
)

tasks["implement_auth_middleware"] = Task(
    "implement_auth_middleware",
    "Implement Bearer Token Auth Middleware",
    "Implement `bearer_auth_middleware` in `crates/agentloop/src/auth/middleware.rs`. Use Actix-web middleware structure (e.g., implementing `Transform`, `Service`). This should verify a bearer token read from environment variable `AUTH_TOKEN`. Add `pub mod middleware;` to `crates/agentloop/src/auth/mod.rs`. Adhere to `rust_guidelines`.",
    patterns=impl_patterns_base + [
        rp("crates/agentloop/src/auth/middleware.rs"),
        rp("crates/agentloop/src/auth/mod.rs")
    ]
)

tasks["implement_http_handlers"] = Task(
    "implement_http_handlers",
    "Implement HTTP Endpoint Handlers (stubs)",
    "Create stub implementations for HTTP handlers (`start_research`, `get_status`, `post_message`, `terminate_session`, `conversation_stream`) in `crates/agentloop/src/handlers/`. Create separate files for each (e.g., `start_research.rs`, `get_status.rs`). Use types from `crate::types::*`. Reference `function_signatures.md`. Focus on function signatures, extracting parameters (Path, Json), and returning basic `impl Responder` (e.g., `HttpResponse::Ok().finish()`). Add `pub mod` entries to `crates/agentloop/src/handlers/mod.rs`. Adhere to `rust_guidelines`.",
    patterns=impl_patterns_base + [
        rp("crates/agentloop/src/handlers/mod.rs"),
        rp("crates/agentloop/src/handlers/**/*.rs") # Indicate modification within handlers dir
    ]
)

tasks["implement_session_management"] = Task(
    "implement_session_management",
    "Implement Core Session Management functions",
    "Implement `create_session`, `get_session_mut`, `get_session`, `update_status`, `add_conversation_entry`, `add_context_entry` in `crates/agentloop/src/session/manager.rs`. Use `SessionData`, `AppState`, `SessionStatus`, etc. Define logic to interact with `AppState`'s session map (likely a `DashMap` or `RwLock<HashMap>`). Add `pub mod manager;` to `crates/agentloop/src/session/mod.rs`. Adhere to `rust_guidelines`.",
    patterns=impl_patterns_base + [
        rp("crates/agentloop/src/session/manager.rs"),
        rp("crates/agentloop/src/session/mod.rs")
    ]
)

tasks["implement_tool_dispatch"] = Task(
    "implement_tool_dispatch",
    "Implement Tool Registration & Dispatch",
    "Implement `register_tools` and `dispatch_tools` in `crates/agentloop/src/tools/dispatch.rs`. Define the `ToolHandler` type alias (`type ToolHandler = fn(...) -> Pin<Box<dyn Future<Output = Result<...>>>>;`). `register_tools` should return a `HashMap<String, ToolHandler>`. Initially, return an empty map. `dispatch_tools` should handle looking up and calling a tool handler based on `ToolChoice`. Add `pub mod dispatch;` to `crates/agentloop/src/tools/mod.rs`. Reference `api_clients` crate structure if available. Adhere to `rust_guidelines`.",
    patterns=impl_patterns_base + [
        rp("crates/agentloop/src/tools/dispatch.rs"),
        rp("crates/agentloop/src/tools/mod.rs"),
        rp("crates/api_clients/src/lib.rs"), # Reference external tools if needed
    ]
)

tasks["implement_conversation_logic"] = Task(
    "implement_conversation_logic",
    "Implement Conversation & Prompt Management",
    "Implement `build_llm_prompt` and `conversation_event_stream` in `crates/agentloop/src/conversation/prompt.rs` and `crates/agentloop/src/conversation/stream.rs` respectively. `build_llm_prompt` should construct a string or message list from `SessionData`. `conversation_event_stream` should return a stream (`impl Stream<Item = ...>`). Create basic structures. Add `pub mod prompt;` and `pub mod stream;` to `crates/agentloop/src/conversation/mod.rs`. Adhere to `rust_guidelines`.",
    patterns=impl_patterns_base + [
        rp("crates/agentloop/src/conversation/prompt.rs"),
        rp("crates/agentloop/src/conversation/stream.rs"),
        rp("crates/agentloop/src/conversation/mod.rs"),
    ]
)

tasks["implement_background_tasks"] = Task(
    "implement_background_tasks",
    "Implement Background Tasks (stubs)",
    "Create stubs for `background_evaluator_task` and `timeout_task` in `crates/agentloop/src/background/tasks.rs`. Implement the basic async function structure (e.g., `async fn background_evaluator_task(app_state: web::Data<AppState>) { loop { ... } }`). Leave evaluation/compaction logic minimal for now (e.g., print messages, sleep). Add `pub mod tasks;` to `crates/agentloop/src/background/mod.rs`. Adhere to `rust_guidelines`.",
    patterns=impl_patterns_base + [
        rp("crates/agentloop/src/background/tasks.rs"),
        rp("crates/agentloop/src/background/mod.rs")
    ]
)

tasks["implement_context_eval_compaction"] = Task(
    "implement_context_eval_compaction",
    "Implement Context Evaluation & Compaction (stubs)",
    "Create stubs for `evaluate_context`, `check_termination`, `should_compact_history`, `compact_history`, `summarize_entries` in `crates/agentloop/src/conversation/compaction.rs`. Implement function signatures based on `function_signatures.md`, return default values (e.g., `false`, `None`). Add `pub mod compaction;` to `crates/agentloop/src/conversation/mod.rs`. Adhere to `rust_guidelines`.",
    patterns=impl_patterns_base + [
        rp("crates/agentloop/src/conversation/compaction.rs"),
        rp("crates/agentloop/src/conversation/mod.rs")
    ]
)

tasks["implement_websocket_management"] = Task(
    "implement_websocket_management",
    "Implement WebSocket Management",
    "Implement `register_ws_recipient` and `broadcast_event` in `crates/agentloop/src/websocket/manager.rs`. `register_ws_recipient` should store recipient channels (e.g., `mpsc::Sender<WebsocketEvent>`) associated with a `SessionId` in `AppState`. `broadcast_event` should retrieve relevant recipients and send the event. Add `pub mod manager;` to `crates/agentloop/src/websocket/mod.rs`. Adhere to `rust_guidelines`.",
    patterns=impl_patterns_base + [
        rp("crates/agentloop/src/websocket/manager.rs"),
        rp("crates/agentloop/src/websocket/mod.rs")
    ]
)

# 5. Web Layer (Actix)
tasks["setup_web_server"] = Task(
    "setup_web_server",
    "Setup Actix Web Server in main.rs",
    "Modify `crates/agentloop/src/main.rs` to initialize and run an Actix-web server. Use `init_app_state` to create `AppState`. Use `HttpServer::new` with an app factory. Call a routing function (e.g., `config::configure_routes`) inside the factory. Apply the `bearer_auth_middleware` using `.wrap()`. Start background tasks (`background_evaluator_task`, `timeout_task`) using `tokio::spawn`. Adhere to `rust_guidelines`.",
    patterns=impl_patterns_base + [
        rp("crates/agentloop/src/main.rs"),
        rp("crates/agentloop/src/auth/middleware.rs"),
        rp("crates/agentloop/src/background/tasks.rs"),
        rp("crates/agentloop/src/state/mod.rs"), # For init_app_state usage
        rp("crates/agentloop/src/config/routes.rs"), # Context for configure_routes fn
        rp("crates/agentloop/src/config/mod.rs"), # Module context
    ]
)

tasks["define_web_routes"] = Task(
    "define_web_routes",
    "Define Web Routes",
    "In `crates/agentloop/src/config/routes.rs`, define an Actix-web configuration function (e.g., `pub fn configure_routes(cfg: &mut web::ServiceConfig)`) mapping paths from `brief_v2.md` (e.g., `/research`, `/research/{session_id}/status`) to the corresponding handlers in `crate::handlers::*`. Include the WebSocket route (`/research/{session_id}/ws`) pointing to a WebSocket handler function (`crate::websocket::handler::ws_handler`). Add `pub mod routes;` to `crates/agentloop/src/config/mod.rs`. Adhere to `rust_guidelines`.",
    patterns=impl_patterns_base + [
        BRIEF_V2_FILE, # Source for paths
        rp("crates/agentloop/src/handlers/**/*.rs"), # Need handler signatures
        rp("crates/agentloop/src/websocket/handler.rs"), # Need WS handler signature
        rp("crates/agentloop/src/config/routes.rs"),
        rp("crates/agentloop/src/config/mod.rs"),
    ]
)

tasks["implement_websocket_handler"] = Task(
    "implement_websocket_handler",
    "Implement WebSocket Connection Handler",
    "Implement the Actix-web WebSocket handler function (e.g., `ws_handler`) in `crates/agentloop/src/websocket/handler.rs`. This function should use `actix_web_actors::ws`. It needs to handle the WebSocket lifecycle (`started`, `handle message`, `finished`). Parse incoming messages as `WebsocketRequest`, interact with session management (`get_session_mut`, `add_conversation_entry`) and broadcasting (`register_ws_recipient`, `broadcast_event`). Send `WebsocketEvent` messages back to the client. Add `pub mod handler;` to `crates/agentloop/src/websocket/mod.rs`. Adhere to `rust_guidelines`.",
    patterns=impl_patterns_base + [
        rp("crates/agentloop/src/websocket/manager.rs"), # For register/broadcast
        rp("crates/agentloop/src/session/manager.rs"), # For session interaction
        rp("crates/agentloop/src/websocket/handler.rs"),
        rp("crates/agentloop/src/websocket/mod.rs"),
    ]
)

# 6. Basic UI (Placeholders)
tasks["setup_ui_serving"] = Task(
    "setup_ui_serving",
    "Configure web server to serve static UI files",
    "Modify the Actix-web route configuration (`crates/agentloop/src/config/routes.rs`) to serve static files from the `crates/agentloop/ui/` directory using `actix_files::Files`. Mount it at a suitable path like `/ui` or `/`.",
    patterns=[
        rp("crates/agentloop/src/config/routes.rs"),
        rp("crates/agentloop/ui/"), # Indicate UI dir exists as context/target
        RUST_GUIDELINES_FILE # General guidelines
        ]
)

tasks["implement_basic_chat_ui"] = Task(
    "implement_basic_chat_ui",
    "Implement basic HTML/JS chat UI",
    "Create a simple HTML structure in `crates/agentloop/ui/index.html` with an input field, a send button, and a message display area. Create `crates/agentloop/ui/app.js` with basic JavaScript to: 1. Establish a WebSocket connection to the `/research/{session_id}/ws` endpoint (hardcode a session ID or get one). 2. Send `WebsocketRequest::Research` or `WebsocketRequest::Message` (as JSON) on button click. 3. Display incoming `WebsocketEvent` messages (JSON) in the display area.",
    patterns=[
        rp("crates/agentloop/src/types/ws_request.rs"), # Structure for sending
        rp("crates/agentloop/src/types/ws_event.rs"), # Structure for receiving
        rp("crates/agentloop/ui/index.html"),
        rp("crates/agentloop/ui/app.js")
        ]
)

# 7. CLI (Minimal stub)
tasks["create_cli_main"] = Task(
    "create_cli_main",
    "Create CLI entry point (stub)",
    "Create `crates/agentloop/src/bin/cli.rs`. Add a simple `main` function that prints a placeholder message (e.g., \"Agentloop CLI - Not implemented yet\"). Add `[[bin]]\nname = \"agentloop-cli\"\npath = \"src/bin/cli.rs\"` to `crates/agentloop/Cargo.toml`.",
    patterns=[
        rp("crates/agentloop/src/bin/cli.rs"),
        rp("crates/agentloop/Cargo.toml"),
        RUST_GUIDELINES_FILE
    ]
)


# --- Define Dependencies ---
# (Dependency logic remains the same as it works on task IDs)
dependency_map = {
    "define_common_types": ["create_dirs"],
    **{f"define_{fid}": ["create_dirs", "define_common_types"] for fid, _, _, in data_structure_files},
    "implement_app_bootstrap": ["create_dirs", "define_app_state"],
    "implement_auth_middleware": ["create_dirs"],
    "implement_http_handlers": ["create_dirs"] + [f"define_{fid}" for fid, _, _, in data_structure_files],
    "implement_session_management": ["create_dirs", "define_app_state", "define_session_data"] + [f"define_{fid}" for fid in ["session_status", "conversation_entry", "context_entry"]],
    "implement_tool_dispatch": ["create_dirs", "define_tool_choice"],
    "implement_conversation_logic": ["create_dirs", "define_session_data"],
    "implement_background_tasks": ["create_dirs", "define_app_state"],
    "implement_context_eval_compaction": ["create_dirs", "define_conversation_entry", "define_context_entry", "define_compaction_policy"],
    "implement_websocket_management": ["create_dirs", "define_app_state", "define_ws_event", "define_ws_request"],
    # Web server setup depends on bootstrap, auth, background tasks, and route config *function* being defined
    # Route *definition* depends on handlers, so setup depends on route *definition task* implicitly through explicit dep
    "setup_web_server": ["implement_app_bootstrap", "implement_auth_middleware", "implement_background_tasks", "define_web_routes"],
    # Route definitions depend on handlers and WS handler implementations being available
    "define_web_routes": ["create_dirs", "implement_http_handlers", "implement_websocket_handler"],
    # WS handler depends on WS management, session management
    "implement_websocket_handler": ["create_dirs", "implement_websocket_management", "implement_session_management"],
    # UI serving depends on dir creation and route definitions (where serving is configured)
    "setup_ui_serving": ["create_dirs", "define_web_routes"],
    # Basic UI implementation depends on WS handler being defined (for endpoint) and UI files being served
    "implement_basic_chat_ui": ["implement_websocket_handler", "setup_ui_serving"],
    # CLI depends on dirs
    "create_cli_main": ["create_dirs"],
    # Make sure specific background/conversation logic depends on stubs being created first
    "implement_background_tasks": ["implement_context_eval_compaction"], # Evaluator task needs eval logic defined
}

# Apply dependencies from the map
for dependent, dependencies in dependency_map.items():
    if dependent in tasks:
        for dependency in dependencies:
            if dependency in tasks:
                tasks[dependent].dependencies.add(dependency)
                tasks[dependency].dependents.add(dependent)
            else:
                print(f"Warning: Dependency '{dependency}' for task '{dependent}' not found. Skipping.")
    else:
        print(f"Warning: Dependent task '{dependent}' not found in tasks list. Skipping dependencies.")


# --- Execution Logic ---
# (Execution logic remains the same as it operates on Task objects and dependencies)
async def run_task(task: Task):
    """Runs a single task using asyncio.create_subprocess_shell."""
    print(f"🟡 Starting task: {task.id} - {task.description}")
    # Log the command safely, escaping potential issues for printing
    log_cmd = task.command.replace('"', '\"') # Basic escaping for printing
    print(f"   Running command (truncated): {log_cmd[:300]}...") # Increased truncation length

    # Using create_subprocess_shell (use with caution if inputs aren't controlled)
    proc = await asyncio.create_subprocess_shell(
        task.command,
        stdout=asyncio.subprocess.PIPE,
        stderr=asyncio.subprocess.PIPE,
        cwd=PROJECT_ROOT # Run commands from project root
    )
    stdout, stderr = await proc.communicate()
    stdout_str = stdout.decode().strip()
    stderr_str = stderr.decode().strip()

    if proc.returncode == 0:
        print(f"🟢 Finished: {task.id}")
        if stdout_str:
            # Limit output length for clarity
            print(f"   stdout (truncated):\n{stdout_str[:500]}{'...' if len(stdout_str) > 500 else ''}")
        if stderr_str: # Sometimes tools output non-fatal info to stderr
             print(f"   stderr (possibly info, truncated):\n{stderr_str[:500]}{'...' if len(stderr_str) > 500 else ''}")
        return True # Indicate success
    else:
        print(f"🔴 Failed: {task.id} (Return Code: {proc.returncode})")
        if stdout_str:
            print(f"   stdout:\n{stdout_str}")
        if stderr_str:
            print(f"   stderr:\n{stderr_str}")
        return False # Indicate failure

async def execute_dag():
    """Executes the DAG of tasks respecting dependencies."""
    completed = set()
    failed = set()
    ready = deque([tid for tid, t in tasks.items() if not t.dependencies])
    in_progress = set()

    print(f"🚀 Starting DAG execution. Initial ready tasks: {[t for t in ready]}")

    while ready or in_progress:
        runnable_tasks = []
        processed_in_queue = set() # Avoid infinite loops if deps are tricky

        # Drain the ready queue into runnable_tasks if dependencies are met
        queue_len = len(ready)
        for _ in range(queue_len):
            tid = ready.popleft()
            if tid in completed or tid in failed or tid in in_progress or tid in processed_in_queue:
                continue # Skip tasks already handled or processed this cycle

            processed_in_queue.add(tid)
            if all(dep in completed for dep in tasks[tid].dependencies):
                runnable_tasks.append(tid)
            else:
                # Dependencies not met, put back at the end of the queue for later check
                ready.append(tid)


        if not runnable_tasks:
            if not in_progress:
                # No tasks are runnable, and nothing is in progress. Check final state.
                total_processed = len(completed) + len(failed)
                if total_processed == len(tasks):
                    print("\n✅ DAG Execution finished.")
                else:
                    print("\n❌ DAG Execution stalled.")
                    print(f"   Completed: {len(completed)}, Failed: {len(failed)}, Total: {len(tasks)}")
                    # Find tasks that are waiting for failed dependencies
                    waiting_for_failed = defaultdict(list)
                    still_waiting = []
                    potential_cycle_tasks = set()

                    for tid, t in tasks.items():
                        if tid not in completed and tid not in failed and tid not in in_progress:
                            failed_deps = t.dependencies.intersection(failed)
                            met_deps = t.dependencies.intersection(completed)
                            waiting_deps = t.dependencies - met_deps - failed

                            if failed_deps:
                                waiting_for_failed[tid].extend(list(failed_deps))
                            elif waiting_deps:
                                 still_waiting.append(tid)
                                 potential_cycle_tasks.add(tid)


                    if waiting_for_failed:
                        print("   Tasks waiting on failed dependencies:")
                        for tid, failed_deps in waiting_for_failed.items():
                            print(f"     - {tid} (waits for: {', '.join(failed_deps)})")
                    if still_waiting:
                         print(f"   Tasks still waiting for pending dependencies: {still_waiting}")

                    # Basic cycle check
                    if potential_cycle_tasks:
                        all_waiting_deps = set()
                        for tid in potential_cycle_tasks:
                            all_waiting_deps.update(tasks[tid].dependencies - completed - failed)

                        if all_waiting_deps.issubset(potential_cycle_tasks):
                            print(f"   🔥 Potential dependency cycle detected involving tasks: {potential_cycle_tasks}")


                break # Exit main loop
            else:
                # Nothing runnable, but tasks are in progress. Wait.
                await asyncio.sleep(0.2)
                continue

        print(f"\n🚀 Executing batch of {len(runnable_tasks)} tasks: {', '.join(runnable_tasks)}")
        for tid in runnable_tasks:
            in_progress.add(tid)

        # Run the batch concurrently
        results = await asyncio.gather(*[run_task(tasks[tid]) for tid in runnable_tasks], return_exceptions=True)

        # Process results and update task states
        newly_completed = set()
        for tid, result in zip(runnable_tasks, results):
            in_progress.remove(tid)
            if isinstance(result, Exception):
                print(f"💥 Task {tid} CRASHED with exception: {result}")
                failed.add(tid)
            elif result is True: # Task reported success (run_task returned True)
                if tid not in failed: # Ensure it didn't fail silently before
                    completed.add(tid)
                    newly_completed.add(tid)
            else: # Task reported failure (run_task returned False or crashed)
                failed.add(tid)

        # Add dependents of newly completed tasks to the ready queue if all their dependencies are now met
        if newly_completed:
            print(f"✨ Tasks completed in this batch: {newly_completed}")
            potential_new_ready = set()
            for tid in newly_completed:
                potential_new_ready.update(tasks[tid].dependents)

            added_to_ready = 0
            for dep_id in potential_new_ready:
                # Check if not already done/failed/running and if deps are met
                if dep_id not in completed and dep_id not in failed and dep_id not in in_progress:
                    if all(d in completed for d in tasks[dep_id].dependencies):
                        # Check not already in queue to avoid duplicates if logic allows
                        if dep_id not in ready:
                           ready.append(dep_id)
                           added_to_ready +=1
            if added_to_ready > 0:
                 print(f"   Added {added_to_ready} new tasks to the ready queue.")

    # Final Summary
    if failed:
        print(f"\n❌ DAG execution finished with {len(failed)} failed tasks: {failed}")
    elif len(completed) == len(tasks):
        print("\n✅ All tasks completed successfully.")
    else:
         # This case should be caught by the stall detection, but as a fallback:
         print(f"\n⚠️ DAG execution finished, but not all tasks completed. Completed: {len(completed)}, Failed: {len(failed)}, Total: {len(tasks)}")


if __name__ == "__main__":
    # Basic validation before running
    if not os.path.exists(RUST_GUIDELINES_FILE):
        print(f"Error: Required pattern file not found at {RUST_GUIDELINES_FILE}")
    elif not os.path.exists(BRIEF_V2_FILE):
         print(f"Error: Required pattern file not found at {BRIEF_V2_FILE}")
    elif not os.path.exists(DATA_STRUCTURES_FILE):
         print(f"Error: Required pattern file not found at {DATA_STRUCTURES_FILE}")
    elif not os.path.exists(FUNCTION_SIGNATURES_FILE):
         print(f"Error: Required pattern file not found at {FUNCTION_SIGNATURES_FILE}")
    else:
        asyncio.run(execute_dag())