import asyncio
from collections import defaultdict, deque

class Task:
    def __init__(self, id, command):
        self.id = id
        self.command = command
        self.dependencies = set()
        self.dependents = set()

# Define tasks
tasks = {
    "upload_asset": Task("upload_asset", 'zen do --instruction "Implement Asset Upload Page: drag & drop zone, file picker, upload progress, thumbnail previews" --patterns "frontend/src/**,frontend/src/pages/DashboardCreatives.tsx,backend/src/routes/assets/create_asset.rs"'),
    "gallery_asset": Task("gallery_asset", 'zen do --instruction "Implement Asset Gallery with grid and list view, filters by type, pagination or infinite scroll" --patterns "frontend/src/**,frontend/src/pages/DashboardCreatives.tsx,frontend/src/components/**,backend/src/routes/assets/list_assets.rs"'),
    "detail_asset": Task("detail_asset", 'zen do --instruction "Implement Asset Detail Modal with preview, metadata, download, delete options" --patterns "frontend/src/**,frontend/src/components/**,backend/src/routes/assets/get_asset_by_id.rs"'),
    "bulk_asset": Task("bulk_asset", 'zen do --instruction "Implement Bulk Asset Actions: select multiple assets, delete, download" --patterns "frontend/src/**,frontend/src/pages/DashboardCreatives.tsx,backend/src/routes/assets/delete_asset.rs"'),

    "create_research": Task("create_research", 'zen do --instruction "Implement New Research Form: select type/template, context fields, attach assets/style, submit" --patterns "frontend/src/**,frontend/src/pages/PlaygroundPage.tsx,backend/src/routes/research/create_research.rs"'),
    "dashboard_research": Task("dashboard_research", 'zen do --instruction "Implement Research Dashboard: table view with filters (status/date), search, action buttons" --patterns "frontend/src/**,frontend/src/pages/PlaygroundPage.tsx,frontend/src/components/**,backend/src/routes/research/list_research.rs"'),
    "detail_research": Task("detail_research", 'zen do --instruction "Implement Research Detail View: show structured output (JSON viewer & rendered summaries)" --patterns "frontend/src/**,frontend/src/pages/PlaygroundPage.tsx,backend/src/routes/research/get_research_by_id.rs"'),

    "collections": Task("collections", 'zen do --instruction "Implement Collections Page: create/edit/delete collections, table or card list layout" --patterns "frontend/src/**,frontend/src/pages/DashboardCreatives.tsx,backend/src/routes/collections/**"'),
    "collection_detail": Task("collection_detail", 'zen do --instruction "Implement Collection Detail Page: creative cards with filters (format/date), actions (view/download/favorite)" --patterns "frontend/src/**,frontend/src/pages/DashboardCreatives.tsx,backend/src/routes/creatives/list_creatives.rs"'),
    "creative_form": Task("creative_form", 'zen do --instruction "Implement Creative Generation Form: select style, research, format; preview estimated output; trigger generation" --patterns "frontend/src/**,frontend/src/pages/DashboardCreatives.tsx,backend/src/routes/creatives/create_creative.rs"'),
    "format_mgmt": Task("format_mgmt", 'zen do --instruction "Implement Format Management UI: list, create/edit/delete custom formats, search, filter" --patterns "frontend/src/**,frontend/src/pages/DashboardSettings.tsx,backend/src/routes/formats/**"'),

    "global_search": Task("global_search", 'zen do --instruction "Implement Global Search across Styles, Assets, Research, Creatives" --patterns "frontend/src/**,frontend/src/components/**,backend/src/routes/**/list_*.rs"'),
    "auth": Task("auth", 'zen do --instruction "Implement Authentication & Authorization handling on all routes" --patterns "frontend/src/**,frontend/src/store/authStore.ts,backend/src/middleware/auth.rs"'),
    "ui_states": Task("ui_states", 'zen do --instruction "Implement consistent loading states, error handling, and empty state UI across all pages" --patterns "frontend/src/**,frontend/src/components/**,frontend/src/pages/**"'),
    "theme_toggle": Task("theme_toggle", 'zen do --instruction "Implement optional dark/light theme toggle with persistent setting" --patterns "frontend/src/**,frontend/src/components/**,frontend/index.css"'),
}

# Define dependencies
deps = [
    ("upload_asset", "gallery_asset"),
    ("upload_asset", "detail_asset"),
    ("upload_asset", "bulk_asset"),
    ("create_research", "dashboard_research"),
    ("create_research", "detail_research"),
    ("creative_form", "collection_detail"),
    ("auth", "upload_asset"),
    ("auth", "create_research"),
    ("auth", "creative_form"),
    ("auth", "collections"),
    ("dashboard_research", "global_search"),
    ("gallery_asset", "global_search"),
    ("collection_detail", "global_search"),
]

# Apply dependencies
for before, after in deps:
    tasks[after].dependencies.add(before)
    tasks[before].dependents.add(after)

# Topological execution with async
async def run_task(task: Task):
    print(f"🟡 Starting task: {task.id}")
    proc = await asyncio.create_subprocess_shell(
        task.command,
        stdout=asyncio.subprocess.PIPE,
        stderr=asyncio.subprocess.PIPE,
    )
    stdout, stderr = await proc.communicate()
    if proc.returncode == 0:
        print(f"🟢 Finished: {task.id}")
    else:
        print(f"🔴 Failed: {task.id}\n{stderr.decode()}")

async def execute_dag():
    ready = deque([tid for tid, t in tasks.items() if not t.dependencies])
    in_progress = set()
    completed = set()

    while ready or in_progress:
        current_batch = list(ready)
        ready.clear()

        coros = [run_task(tasks[tid]) for tid in current_batch]
        in_progress.update(current_batch)
        await asyncio.gather(*coros)

        for tid in current_batch:
            completed.add(tid)
            in_progress.remove(tid)
            for dep in tasks[tid].dependents:
                tasks[dep].dependencies.remove(tid)
                if not tasks[dep].dependencies and dep not in completed:
                    ready.append(dep)

asyncio.run(execute_dag())
