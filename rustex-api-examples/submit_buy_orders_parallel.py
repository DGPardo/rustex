import aiohttp
import asyncio
import requests
import os
from dotenv import load_dotenv
from time import time
from random import random
import numpy as np

# Limit concurrency
CONCURRENT_REQUESTS = 100

# Set the number of parallel requests
NUMBER_OF_REQUESTS = 10_000

async def send_requests(session, exchange_id, cert_path, key_path, headers, semaphore):
    print(exchange_id)
    async with semaphore:  # Limit number of concurrent requests
        url = "https://0.0.0.0:5000/v1/orders"

        start = time()

        try:
            # Set a timeout for requests
            timeout = aiohttp.ClientTimeout(total=10)  # 10 seconds timeout
            quantity = random() * 1_000_000
            price = int(random() * 1_000_000)
            order_type = "buy" if random() < 0.5 else "sell"
            if random() < 0.33333:
                exchange = "BTC_USD"
            elif random() < 0.66666:
                exchange = "BTC_EUR"
            else:
                exchange = "BTC_GBP"
            response_buy = await session.post(url, json={
                "price": price,
                "quantity": quantity,
                "exchange": exchange,
                "orderType": order_type,
            }, timeout=timeout, headers=headers)

        except asyncio.TimeoutError:
            print(f"Exchange {exchange_id}: Request timed out!")
        except Exception as e:
            print(f"Exchange {exchange_id}: Error - {e}")
    return time() - start


async def main():
    load_dotenv()
    cert_path = os.getenv("TLS_CERT_PATH")
    key_path = os.getenv("TLS_KEY_PATH")

    url = "https://0.0.0.0:5000/v1/public/auth/login"
    response = requests.post(url, cert=(cert_path, key_path), verify=False, json={
        "username": "foo",
        "hashed_password": "bar",
    })

    if not response.ok:
        print("Failed to login in the system")
        exit(1)
    bearer_token = response.text
    headers = {
        "Authorization": f"Bearer {bearer_token}",
        "Content-Type": "application/json"
    }

    connector = aiohttp.TCPConnector(ssl=False)
    semaphore = asyncio.Semaphore(CONCURRENT_REQUESTS)  # Control concurrency

    start = time()
    async with aiohttp.ClientSession(connector=connector) as session:
        tasks = [
            send_requests(session, exchange_id, cert_path, key_path, headers, semaphore)
            for exchange_id in range(NUMBER_OF_REQUESTS)
        ]
        latencies = await asyncio.gather(*tasks)
    elapsed = time() - start
    p95_latency = np.percentile(latencies, 95)

    print(f"{NUMBER_OF_REQUESTS} requests took: {elapsed}. P95 Latency {p95_latency}")

# Run the async event loop
if __name__ == "__main__":
    asyncio.run(main())
