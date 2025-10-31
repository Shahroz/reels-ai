## Blueprint for any new feature involving changes from db to frontend (instruction for a LLM-user interaction based on experience):
1. Write a migration and corresponding model in backend/db and corresponding function in backend/routes
2. Run a migration
3. Check if DB has correct columns
4. Write test on the BE to check if a function and model correctly insert data into DB. Check in DB that the data is there.
5. Include the model and function in openapi.rs
6. Regenerate the api-client using make generate-api-client
7. Check that Services and Models were correctly automatically generated in the frontend
8. Run make dev to regenerate tanstack routes.
9. When creating new components, pass /features/ as entire catalogue to context -> it helps LLMs build an understanding of the repo logic




Typical problems and how to solve them:
- make sure to use use crate::auth::tokens::Claims; for authentication in backend/routes (LLM have tendencies to come up with their own authentication)
-