FROM node:26-slim AS build
COPY --from=oven/bun:1.3.8 /usr/local/bin/bun /usr/local/bin/bun
WORKDIR /app
COPY package.json bun.lock ./
COPY web/package.json web/
COPY desktop/package.json desktop/
COPY packages/ui/package.json packages/ui/
RUN bun install --frozen-lockfile --filter web
COPY packages/ui packages/ui
COPY web web
ARG SITE_VERSION=dev
RUN cd web && VITE_SITE_VERSION=$SITE_VERSION bun run build

FROM node:26-slim
WORKDIR /app
COPY --from=build /app/web/package.json ./
COPY --from=build /app/web/build ./build
ENV NODE_ENV=production PORT=3000 API_URL=http://api:8080
EXPOSE 3000
USER node
CMD ["node", "build"]
