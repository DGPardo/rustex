import requests
import json
from time import time
import os
from dotenv import load_dotenv

load_dotenv()
cert_path = os.getenv("TLS_CERT_PATH")
key_path = os.getenv("TLS_KEY_PATH")

print(cert_path)
print(key_path)


url = "https://0.0.0.0:5000/v1/public/login"
response = requests.post(url, cert=(cert_path, key_path), verify=False, json={
    "username": "diego",
    "hashed_password": "0000",
})

if not response.ok:
    print("Failed to login in the system")
    exit(1)

bearer_token = response.text
print("Got Bearer token: ", bearer_token)

headers = {
    "Authorization": f"Bearer {bearer_token}",
}

url = "https://0.0.0.0:5000/v1/orders/buy"

start = time()
response = requests.post(url, cert=(cert_path, key_path), verify=False, json={
    "price": 1,
    "quantity": 177.894,
    "exchange": "BTC_USD",
}, headers=headers)

print(response.ok)
print(response.content)
print(time() - start)

url = "https://0.0.0.0:5000/v1/orders/sell"
response = requests.post(url, cert=(cert_path, key_path), verify=False, json={
    "price": 1,
    "quantity": 177.894,
    "exchange": "BTC_USD",
}, headers=headers)

print(response.ok)
print(time() - start)
