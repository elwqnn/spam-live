FROM node:26-slim AS build
COPY --from=oven/bun:1.3.8 /usr/local/bin/bun /usr/local/bin/bun
WORKDIR /app
COPY package.json bun.lock ./
RUN bun install --frozen-lockfile
COPY . .
RUN bun run build

FROM node:26-slim
WORKDIR /app
COPY --from=build /app/package.json ./
COPY --from=build /app/build ./build
ENV NODE_ENV=production PORT=3000 API_URL=http://api:8080
EXPOSE 3000
USER node
CMD ["node", "build"]
