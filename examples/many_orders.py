import aiohttp
import asyncio
import requests
import ssl
import os
from dotenv import load_dotenv
from time import time
from random import random
import numpy as np

# Limit concurrency
CONCURRENT_REQUESTS = 100

# Set the number of parallel requests
NUMBER_OF_REQUESTS = 10_000

async def send_requests(session, request_number, semaphore, headers):
    async with semaphore:  # Limit number of concurrent requests
        url = "https://127.0.0.1:5000/v1/orders"

        start = time()
        try:
            quantity = random() * 1_000_000
            price = int(random() * 1_000_000)
            order_type = "buy" if random() < 0.5 else "sell"

            if random() < 0.33333:
                exchange = "BTC_USD"
            elif random() < 0.66666:
                exchange = "BTC_EUR"
            else:
                exchange = "BTC_GBP"

            response = await session.post(url, headers=headers, json={
                "price": price,
                "quantity": quantity,
                "exchange": exchange,
                "orderType": order_type,
            })
            print(f"[Completed] Order {request_number}")

        except asyncio.TimeoutError:
            print(f"Exchange {request_number}: Request timed out!")
        except Exception as e:
            print(f"Exchange {request_number}: Error - {e}")
    return time() - start


async def login(url, ssl_context):
    # Create a connector using the SSL context
    connector = aiohttp.TCPConnector(ssl=ssl_context)

    async with aiohttp.ClientSession(connector=connector) as session:
        async with session.post(
            url,
            json={
                "username": "foo",
                "pass": "bar",
            }
        ) as response:
            return await response.text()  # Read response text (or use .json() if expecting JSON)


async def main():
    load_dotenv()
    cert_path = os.getenv("TLS_CERT_PATH")
    key_path = os.getenv("TLS_KEY_PATH")
    ca_cert = os.getenv("TLS_CA_PATH")

    ssl_context = ssl.create_default_context(cafile=ca_cert)  # CA Verification
    ssl_context.load_cert_chain(certfile=cert_path, keyfile=key_path)  # Client cert + key

    print(
    f"""

    Using the following certificates
        - certificate: {cert_path}
        - key: {key_path}
        - ca: {ca_cert}

    """)

    url = "https://127.0.0.1:5000/v1/public/auth/login"
    bearer_token = await login(url, ssl_context)
    headers = {
        "Authorization": bearer_token,
        "Content-Type": "application/json"
    }

    connector = aiohttp.TCPConnector(ssl=ssl_context)
    semaphore = asyncio.Semaphore(CONCURRENT_REQUESTS)  # Control concurrency

    start = time()
    async with aiohttp.ClientSession(connector=connector) as session:
        tasks = [
            send_requests(session, exchange_id, semaphore, headers)
            for exchange_id in range(NUMBER_OF_REQUESTS)
        ]
        latencies = await asyncio.gather(*tasks)
    elapsed = (time() - start) * 1000

    p95_latency = np.percentile(latencies, 95)

    print(
    f"""

    Summary of results
        - All times are measured in the client-side
        - All times are expressed in miliseconds

    Total number of requests: {NUMBER_OF_REQUESTS}
    Concurrent requests limit: {CONCURRENT_REQUESTS}
    Total Elapsed time: {elapsed:.3f} [ms]

    Average time: {elapsed / NUMBER_OF_REQUESTS:.3f} [ms] / Per Request
    StdDev : {np.std(elapsed):.3f} [ms]

    P50 Latency: {np.percentile(latencies, 50):.3f} [ms]
    P95 Latency: {np.percentile(latencies, 95):.3f} [ms]
    P99 Latency: {np.percentile(latencies, 99):.3f} [ms]
    MAX Latency: {np.max(latencies):.3f} [ms]

    """)

if __name__ == "__main__":
    asyncio.run(main())
