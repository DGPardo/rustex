import requests
import json
import os
from dotenv import load_dotenv
from random import random

load_dotenv()
cert_path = os.getenv("TLS_CERT_PATH")
key_path = os.getenv("TLS_KEY_PATH")
ca_cert = os.getenv("TLS_CA_PATH")

print(
f"""

Using the following certificates
    - certificate: {cert_path}
    - key: {key_path}
    - ca: {ca_cert}

""")

req_kwargs = {
    "cert": (cert_path, key_path),
    "verify": ca_cert
}


url = "https://127.0.0.1:5000/v1/public/auth/login"
response = requests.post(url, **req_kwargs, json={
    "username": "foo",
    "pass": "bar",
})

assert response.ok, "Failed to get Bearer token"

bearer_token = response.text
req_kwargs['headers'] = {
    "Authorization": bearer_token,
    "Content-Type": "application/json"
}

url = "https://127.0.0.1:5000/v1/orders"

####
#### Create a new order
####
response = requests.post(url, **req_kwargs, json={
    "price": 1,
    "quantity": random() * 1_000_000,
    "exchange": "BTC_USD",
    "orderType": "buy",
})
assert response.ok, 'Failed to execute buy transaction'
order_id = response.text

####
#### Get All User Orders
####
response = requests.get(url, **req_kwargs)
assert response.ok, 'Failed to retrieve my orders'
assert 'BTC_USD' in response.text and order_id in response.text

####
#### Check User Order is pending
####
url = f"https://127.0.0.1:5000/v1/BTC_USD/{order_id}"
response = requests.get(url, **req_kwargs)
assert response.ok, 'Failed to get my order state'
(is_pending, remaining) = response.json() # bool
assert is_pending, "Order is not pending"

####
#### Delete User Order
####
url = f"https://127.0.0.1:5000/v1/BTC_USD/{order_id}"
response = requests.delete(url, **req_kwargs)
assert response.ok, 'Failed to delete my order'
assert response.json()
is_deleted = response.json() # bool
assert is_deleted, "Order could not be deleted"

####
#### Check User Order is NOT pending
####
url = f"https://127.0.0.1:5000/v1/BTC_USD/{order_id}"
response = requests.get(url, **req_kwargs)
assert response.ok, 'Failed to get my order state'
(is_pending, remaining) = response.json() # bool
assert not is_pending, "Order is pending after deletion"
