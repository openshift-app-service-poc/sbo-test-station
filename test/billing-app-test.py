#!/usr/bin/env python3

import json
import sys

import requests

microservice_hosts = ["billing", "auth", "logging"]

station_test_map = {"test": "regression", "stage": "stress", "prod": "smoke"}

def _make_http_request(method, url, body=None, params={}, headers={}, verbose=False):
    method_map = {"get": requests.get,
                  "post": requests.post,
                  "put": requests.put,
                  "delete": requests.delete,
                  "patch": requests.patch}
    request_method = method_map[method]
    response = request_method(url, params=params, headers=headers, json=body)
    if verbose:
        print(json.dumps(headers, indent=4, sort_keys=True))
        print(json.dumps(body, indent=4, sort_keys=True))
        print(json.dumps(params, indent=4, sort_keys=True))
        print(response.text)
    return response.status_code, response.text


def run_test(station):
    if station not in station_test_map:
        print(f"Test Failled, invalid station passed: {station}. Valid stations are test, stage and prod")
        sys.exit(1)

    for host in microservice_hosts:

        test_url = f"http://{host}:8080/{station_test_map[station]}"
        print(f"Testing microservice {host}")
        try:
            status_code, response_text = _make_http_request("get", test_url )
        except Exception as e:
            status_code = 500
            response_text = f"Exception occured: {e}"
        print(test_url)

        if status_code == 200:
            print(f"Test Passed for microservice {host}:")
            print(response_text)
        else:
            print(f"Test Failed for microservice {host}:")
            print(response_text)
            sys.exit(1)


if __name__ == "__main__":
    print("Testing billing app...")
    station = "test"
    if len(sys.argv) > 1:
        station = sys.argv[1]
    run_test(station)
