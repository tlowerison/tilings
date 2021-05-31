FROM rust:1.52.1 AS build-rust

  RUN rustup target add wasm32-unknown-unknown
  RUN [ -z "$(which wasm-pack)" ] && cargo install wasm-pack

  WORKDIR /app
  RUN mkdir -p www
  RUN mkdir -p www/pkg
  COPY ["[^www]/.", "."]
  RUN ls
  RUN cd wasm && wasm-pack build --out-dir ../www/pkg --release

FROM node:8.16 as build-js

  RUN apt-get update && apt-get install -y jq
  COPY www .
  COPY --from=build-rust /app /app
  WORKDIR /app/www
  RUN ls
  RUN yarn --ignore-engines
  RUN yarn add ./pkg
  RUN yarn build

FROM nginx:1.21.0

  COPY nginx /etc/nginx
  COPY --from=build-js /app/www/dist /usr/share/nginx/html

  EXPOSE 3000
  CMD ["nginx", "-g", "daemon off;"]
