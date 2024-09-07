docker build -t paycheck .
docker run -it --rm -v $(pwd):/app -w /app paycheck bash