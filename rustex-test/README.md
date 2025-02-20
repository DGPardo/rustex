# 🦀 rustex-test - End-to-End Testing for Rustex

This crate contains **end-to-end (E2E) tests** for the Rustex exchange system. It interacts with the **rustex-api** service using HTTPS, validating authentication, order placement, retrieval, and deletion.

---

## 🚀 Running the Tests

You can run the tests **either locally** or **inside Docker**.

---

## ✅ 1️⃣ **Run Locally (Standalone)**
The steps to follow are:
```
docker compose --profile debug up  // Wait for the service to finish initialization
docker compose --profile test up
```

Polishing the docker-compose file is WIP to have a smoother
experience.

