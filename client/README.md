# client

Vue 3 + Vite frontend for the fitness app.

## Developing

Install dependencies and start the Vite dev server from this directory:

```sh
npm install
npm run dev
```

The API is proxied to `http://localhost:8080`, so run the Rust server from `../server` as well:

```sh
cargo run
```

## Building

```sh
npm run build
```

Preview the production build with `npm run preview`.

## Testing

```sh
npm test
```

Playwright starts the Rust API and the Vite app, then runs the cardio CRUD e2e test.
