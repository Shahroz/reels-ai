# 1. Use an official Python 3.11 base image.
FROM python:3.11-slim

# 2. Set up a working directory.
WORKDIR /app

# 3. Copy 'requirements.txt' and install the dependencies using pip.
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# 4. Copy the rest of the application source code ('main.py', 'lib.py').
COPY main.py lib.py ./

# 5. Install the 'functions-framework' package.
RUN pip install functions-framework

# 6. Expose port 8080.
EXPOSE 8080

# 7. Set the CMD to run the function using 'functions-framework --target=extract_pans'.
CMD ["functions-framework", "--target=extract_pans"]