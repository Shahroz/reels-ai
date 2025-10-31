 # Expanded Project Brief v2: Four-Section Redesign

 **Date:** 2025-04-21

 ## 1. Introduction

 We are redesigning the application around four core sections—Style, Assets, Research, and Creatives—where each builds upon the previous to deliver final HTML-based creative outputs. Creatives can be grouped in collections and may require multi-resolution support. Assets (images, videos, documents, links, guidelines) will be stored in Google Cloud Storage. This document outlines the scope, technical approach, and initial implementation plan.

 ## 2. Sections Overview

 ### 2.1 Style
 - Persistent style definitions captured from a target website or a user‑provided stylebook.
 - Stored as inline HTML/CSS fragments in the database for reuse in creative generation.
 - Extraction pipeline:
   1. Fetch target URL via Zyte client + HTTP request module.
   2. Parse and normalize CSS/inline styles (existing `style_analysis` module).
   3. Store compressed HTML/CSS in `styles` table.

 ### 2.2 Assets
 - Users upload arbitrary assets (images, video clips, documents, links, brand guidelines).
 - Assets are referenced during Research and Creative phases.
 - All asset files are stored in Google Cloud Storage; metadata (file name, type, URL, owner) lives in the database.

 ### 2.3 Research
 - On‑demand research workflows (“deep research”) scoped to a specific use case or prospect.
 - Integrates with LLM/GenNodes APIs to surface structured data, summaries, or recommendations.
 - Outputs a JSON payload or rich text that feeds into the creative generation process.

 ### 2.4 Creatives
 - Combines Style, selected Assets, and Research outputs to generate one or more HTML‑based creatives (landing pages, social media ads, display banners).
 - Supports multiple formats and resolutions (e.g., 16:9 banner, 1:1 square). Users can choose and iterate on outputs.
 - Creatives are grouped into Collections; each collection tracks versions and user feedback.
 - HTML outputs are stored (and optionally compressed) in the database; visual previews are generated via headless browser screenshots.

 ## 3. Technical Implementation Plan

 ### 3.1 Module Structure (backend/src)
   - services/
     - http_request.rs       – low‑level HTTP client utility
     - gennodes.rs           – GenNodes API integration
     - gcs_storage.rs        – new Google Cloud Storage support
   - style_analysis/         – CSS/HTML parsing and normalization
   - style_cloning/          – style extraction & HTML replication logic
   - research_workflows/     – CRUD and orchestration for research tasks
   - creatives/              – creative generation orchestrator
   - db/                     – tables: styles, assets, research, creatives, collections
   - routes/                 – REST endpoints for each section

 ### 3.2 Assets Module: Google Cloud Storage Support
 - Create `backend/src/services/gcs_storage.rs`:
   ```rust
   use google_cloud_storage::client::Client;
   use google_cloud_storage::http::objects::upload::UploadObjectRequest;
   
   pub struct GcsStorage {
       client: Client,
       bucket: String,
   }

   impl GcsStorage {
       pub async fn new(project_id: &str, bucket: &str) -> anyhow::Result<Self> {
           let client = Client::new().await?;
           Ok(Self { client, bucket: bucket.to_string() })
       }

       pub async fn upload_asset(
           &self,
           data: Vec<u8>,
           object_name: &str,
           content_type: &str,
       ) -> anyhow::Result<String> {
           let req = UploadObjectRequest {
               bucket: &self.bucket,
               name: object_name,
               content_type: Some(content_type.to_string()),
               ..Default::default()
           };
           let object = self.client.upload_object(req, data.into()).await?;
           Ok(object.media_link)
       }

       pub async fn delete_asset(&self, object_name: &str) -> anyhow::Result<()> {
           self.client
               .delete_object(&self.bucket, object_name)
               .await?;
           Ok(())
       }
   }
   ```
 - Environment variables required:
   - `GCS_PROJECT_ID`
   - `GCS_BUCKET_NAME`
   - `GOOGLE_APPLICATION_CREDENTIALS`
 - Add `google-cloud-storage = "^0.9"` to `backend/Cargo.toml`.

 ### 3.3 Style Extraction & Reuse
 - No changes to existing pipeline; new `styles` table will reference GCS URLs if external CSS/images are snapped.

 ### 3.4 Research Workflows
 - Research payload stored in `research` table; integrate into creative orchestrator.

 ### 3.5 Creative Generation & Resolution Handling
 - Generate HTML within a fixed viewport (e.g., 1200×628 px for 16:9; 1080×1080 for 1:1).
 - Use headless browser (e.g., `chromiumoxide` or an external microservice) to capture PNG/JPEG previews.
 - Store preview assets in GCS; metadata in `previews` table.

 ## 4. Data Model Additions
 - styles(id, name, html_content TEXT, created_at)
 - assets(id, owner_id, name, type, gcs_object_name, url, created_at)
 - research(id, owner_id, input JSONB, output JSONB, status, created_at)
 - creatives(id, collection_id, style_id, research_id, html_output TEXT, created_at)
 - collections(id, owner_id, name, metadata JSONB, created_at)
 - previews(id, creative_id, gcs_object_name, url, width, height, created_at)

 ## 5. Questions & Clarifications
 1. What precise aspect ratios and resolutions should we support initially? (e.g., 16:9, 1:1, 9:16)
- answer: we should support common ads formats per platform linkedin ads, facebook ads etc - whatever the format is we should support it - it is rather pixel based
 2. Are assets subject to access control beyond user ownership? Do we need signed URLs or public buckets?
- answer: once the asset is uploaded it is copied to the bucket - assume that the buckets are public
 3. Should previews be public URLs or authenticated downloads?
- answer: public
 4. Do we need versioning or immutability for styles and assets?
- answer: not necessary
 5. Any restrictions on file size or type for uploads?
- answer: 20mb per file (i think it is cloud run limitation - it wont accept bigger requests)
 6. Preferred database migration tool (sqlx-cli, refinery)?
- answer: we are using sqlx right now

 _End of brief v2_

## Style

Style is where we can define the style based on some website style or stylebook defined by the user
The style is something persistent and it is an HTML content in both cases. When we are extracting 
the style from a website it is saved as the inline HTML and later reused to create creatives.

## Assets

The users can upload any type of asset:
- images
- videos
- documents
- links
- guidelines

Assets are used in the Research and Creatives creation.

## Research

Research is our version of deep research which is spawned on demand for a concrete use case.
For example we spawn a research about a prospect to help us create the creatives.

## Creatives

A creative is a combination of all above. We use:
- style
- assets
- research

To create new creatives like:
- landing pages
- visual ads different formats

Creatives can be grouped in collections - each request to create a creative can produce
one or more creatives and the user can choose from them an iterate
