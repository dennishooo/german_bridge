# syntax=docker/dockerfile:1

# Stage 1: Build the application
FROM node:20-slim AS builder

# Set the working directory
WORKDIR /app

# Copy package files first to leverage caching
COPY package.json package-lock.json ./

# Install dependencies - remove package-lock and use npm install to handle optional dependencies
RUN rm -f package-lock.json && npm install

# Copy the rest of the application
COPY . .

# Build the application
RUN npm run build

# Stage 2: Serve with Nginx
FROM nginx:alpine

# Copy custom nginx config
COPY nginx.conf /etc/nginx/nginx.conf

# Copy the build output to nginx html directory
COPY --from=builder /app/build /usr/share/nginx/html

# Expose port 80
EXPOSE 80

# Start nginx
CMD ["nginx", "-g", "daemon off;"]
