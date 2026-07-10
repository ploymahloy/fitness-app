## Data Schema
<img width="776" height="408" alt="Image" src="https://github.com/user-attachments/assets/e4f62f19-4fd4-4ce4-b55d-4f8f2da99cdf" />

## Roadmap

### Standing up the database

- [x] Develop [schema](https://github.com/ploymahloy/fitness-app/blob/master/migrations/20260630041018_init_schema.sql) for SQLite database
- [x] Connect to database via `sqlx-cli` and `sqlite3` and run the migration file (schema ^)

### Learning Rust through CRUD development

- [x] Build API server `/health` [endpoint](https://github.com/ploymahloy/fitness-app/blob/master/src/main.rs#L46-L53) 
- [x] Build 'read' [endpoint](https://github.com/ploymahloy/fitness-app/blob/master/src/main.rs#L99-L114)
- [x] Build 'create [endpoint](https://github.com/ploymahloy/fitness-app/blob/master/src/main.rs#L55-L97)
- [ ] Build 'update' endpoint
- [ ] Build 'delete' endpoint

### UI

- CRUD operations:
  - [ ] Add (create) entry
  - [ ] View (read) entry
  - [ ] Update entry 
  - [ ] Delete entry
- Data display: --> GraphQL?
  - [ ] Statistics view (i.e. average daily protein intake)
  - [ ] Charts (i.e. cardio per week over time)

### Business Logic

- [ ] Calculating this crap ^

### Testing

- Playwright:
  - [ ] Single CRUD test (CRUD, in that order) --> covers majority of app function lol
- Rust:
  - [ ] Database migrations and constraints
  - [ ] API route handlers 
  - [ ] Business logic?
