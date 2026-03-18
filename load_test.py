import asyncio
import aiohttp
import time
import json
import statistics

async def fetch(session, url, key, query_id, semaphore):
    async with semaphore:
        headers = {"Authorization": f"Bearer {key}", "Content-Type": "application/json"}
        # Use a fixed query for half the requests to test caching
        query = "What is Rust?" if query_id % 2 == 0 else f"Load test query {query_id}"
        payload = {"query": query, "max_sources": 5}
        start = time.perf_counter()
        try:
            # Explicitly long timeout for the request itself
            timeout = aiohttp.ClientTimeout(total=60)
            async with session.post(url, headers=headers, json=payload, timeout=timeout) as r:
                latency = (time.perf_counter() - start) * 1000
                return r.status, latency
        except Exception as e:
            latency = (time.perf_counter() - start) * 1000
            return str(e), latency

async def load_test(total_requests, concurrency):
    url = "http://127.0.0.1:4567/v1/search"
    key = "fetchium_1f82fcf3425d0b9999daef4795e3104d032e20aba63a48923ef37d6f0bf22fed"

    print(f"\n--- Load Test: Total={total_requests}, Max Concurrency={concurrency} ---")
    semaphore = asyncio.Semaphore(concurrency)

    # Use a custom connector to ensure we don't hit default connection limits
    connector = aiohttp.TCPConnector(limit=concurrency + 10)

    async with aiohttp.ClientSession(connector=connector) as session:
        tasks = [fetch(session, url, key, i, semaphore) for i in range(total_requests)]
        start = time.perf_counter()
        results = await asyncio.gather(*tasks)
        total_time = (time.perf_counter() - start)

    statuses = [r[0] for r in results]
    latencies = [r[1] for r in results]

    success_count = statuses.count(200)
    error_count = len(statuses) - success_count

    print(f"Total Wall Time: {total_time:.2f}s")
    print(f"Throughput: {total_requests/total_time:.2f} req/s")
    print(f"Success: {success_count}, Errors: {error_count}")

    # Log errors
    errors = [s for s in statuses if s != 200]
    if errors:
        from collections import Counter
        print(f"Error breakdown: {Counter(errors)}")

    if latencies:
        print(f"Avg Latency: {statistics.mean(latencies):.2f}ms")
        print(f"P50 Latency: {statistics.median(latencies):.2f}ms")
        if len(latencies) >= 2:
            print(f"P95 Latency: {statistics.quantiles(latencies, n=20)[18]:.2f}ms")

async def main():
    # Test steady state vs burst
    await load_test(10, 10)
    await asyncio.sleep(5)
    await load_test(50, 10) # 50 requests, 10 at a time
    await asyncio.sleep(5)
    await load_test(20, 20) # 20 requests, 20 at a time

if __name__ == "__main__":
    asyncio.run(main())
