# feat: Add Feed functionality to mobile app (backend + iOS)

## 🎯 Summary
This PR adds a new **Feed** feature where users can publish photos (original + enhanced versions) with AI enhancement prompts. Other users can browse the feed and save prompts to favorites.

## 🔧 What Changed

### Backend
- **Database:** 3 new tables (`feed_posts`, `feed_post_assets`, `favorited_prompts`)
  - Full migrations with proper indexes and constraints
  - Soft delete support for posts
- **Models:** Rust structs for feed entities with proper traits
- **Queries:** CRUD operations for feed posts and favorite prompts
  - Transaction-safe post creation with asset associations
  - Enhancement prompt retrieval from provenance_edges
- **Routes:** 11 new REST API endpoints
  - Feed: create, read, update, delete, list
  - Favorite Prompts: add, list, remove
- **OpenAPI:** Complete specification with all new endpoints and schemas

### Mobile (iOS)
- **Services:** 
  - `FeedService` for API integration with pagination
  - Extended `FavoritesManager` with backend synchronization
- **UI:** 4 new SwiftUI views
  - `FeedView`: main feed with infinite scroll
  - `FeedPostCard`: post display with image, caption, prompts, actions
  - `CreateFeedPostView`: post creation with asset selection
  - `AssetSelectionForFeedView`: multi-select gallery
- **Navigation:** New "Feed" tab in main navigation
- **Share:** Integration from Gallery (multi-select) and Asset Carousel (single)
- **SDK:** Regenerated with 316 updated files

## ⚠️ Known Limitations
Due to Xcode build performance issues with complex SwiftUI:
1. **No image carousel** - shows only first photo from multi-photo posts (badge indicates additional photos)
2. **No thumbnails in CreatePost** - simple ID list instead of image previews
3. **No post editing** - only create and delete (can delete and recreate)

These are build-time issues, not runtime. The backend fully supports carousel and editing.

## 🧪 Testing

### Backend
- ✅ Compiles without errors
- ✅ All migrations run successfully on local database
- ✅ OpenAPI spec validates
- ⚠️ No automated tests yet (manual testing via SDK)

### Mobile
- ✅ Builds in Xcode without errors
- ✅ All new views integrate properly
- ✅ Basic flow tested manually:
  - View feed (empty state + posts)
  - Create post with multiple photos
  - Delete own post
  - Add/remove favorite prompts
  - Share from Gallery and Carousel

## 📝 Deployment Notes

**⚠️ CRITICAL: Database migrations MUST be run on production BEFORE deploying backend!**

### Deployment Steps:

1. **Run Migrations (FIRST):**
```bash
# On production database
export DATABASE_URL="postgresql://user:pass@prod:5432/db"
cd crates/reels/backend
sqlx migrate run
```

Verify:
```sql
-- Check tables exist
SELECT table_name FROM information_schema.tables 
WHERE table_schema = 'public' 
AND table_name IN ('feed_posts', 'feed_post_assets', 'favorited_prompts');
```

2. **Deploy Backend:**
   - Merge PR to main
   - Backend auto-deploys (or manual deploy depending on your setup)
   - Verify health endpoint: `curl https://re.bounti.ai/health`

3. **Verify Feed Endpoints:**
```bash
curl -X GET https://re.bounti.ai/api/feed/posts?page=1&limit=10 \
  -H "Authorization: Bearer TOKEN"
```

4. **Mobile Deployment:**
   - Build from main branch
   - Archive in Xcode
   - Submit to TestFlight
   - Internal testing
   - App Store release

## 📊 Stats

- **18 commits** (5 backend, 1 SDK, 11 mobile, 1 docs cleanup)
- **419 files changed**
  - 41 backend files
  - 62 mobile files (custom code)
  - 316 SDK files (auto-generated)
- **Backend:** +1,200 lines (migrations, models, queries, routes)
- **Mobile:** +900 lines (services, views, integration)

## 🔗 Related

Implements core Feed functionality as requested. Follow-up PRs will address:
- Carousel view (requires refactoring to avoid Xcode build issues)
- Post editing UI
- Thumbnail previews in CreatePost
- Automated tests

## 📸 Screenshots

_[TODO: Add screenshots of:]_
- Feed view (empty state + with posts)
- Create post flow
- Post with enhancement prompt
- Favorite prompt interaction

## ✅ Checklist

- [x] Code compiles without errors
- [x] Migrations tested locally
- [x] OpenAPI spec updated and validates
- [x] SDK regenerated
- [x] Basic functionality tested manually
- [x] No sensitive data in commits
- [x] Documentation removed (Polish dev docs)
- [ ] Automated tests (TODO)
- [ ] Screenshots added (TODO before merge)

---

**Ready for review!** 🚀

