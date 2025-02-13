import requests
import json
from time import time
import os
from dotenv import load_dotenv
from random import random

load_dotenv()
cert_path = os.getenv("TLS_CERT_PATH")
key_path = os.getenv("TLS_KEY_PATH")

print(cert_path)
print(key_path)


url = "https://0.0.0.0:5000/v1/public/auth/login"
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

url = "https://0.0.0.0:5000/v1/orders?order_type=Buy"

start = time()
response = requests.post(url, cert=(cert_path, key_path), verify=False, json={
    "price": 1,
    "quantity": random() * 1_000_000,
    "exchange": "BTC_USD",
}, headers=headers)
assert response.ok, 'Failed to execute buy transaction'
print('Buy transaction took: ', time() - start)

start = time()
url = "https://0.0.0.0:5000/v1/orders?order_type=Sell"
response = requests.post(url, cert=(cert_path, key_path), verify=False, json={
    "price": 1,
    "quantity": random() * 1_000_000,
    "exchange": "BTC_USD",
}, headers=headers)
assert response.ok, 'Failed to execute sell transaction'
print('Sell transaction took: ', time() - start)
