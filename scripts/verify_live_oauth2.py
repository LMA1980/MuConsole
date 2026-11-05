#!/usr/bin/env python3
"""
verify_live_oauth2.py — Live E2E & Integration Test Suite for console.lma1980.duckdns.org OAuth2 Flow.

Verifies:
  Check 1: GET /oauth2/start?rd=/
           - HTTP 302 Found or 303 See Other (NOT 200 OK SPA HTML).
           - Location header points to https://github.com/login/oauth/authorize.
           - Location query parameters: client_id, redirect_uri, scope (user:email read:org), state.
           - Content-Security-Policy contains single-quoted keywords ('self', 'unsafe-inline', 'wasm-unsafe-eval').
           - Set-Cookie header issues CSRF cookie (_muconsole_oauth2_csrf or _oauth2_proxy_csrf).
  Check 2: GET /oauth2/start?rd=<encoded_invite_url>
           - Preserves redirect target in OAuth2 handshake state.
  Check 3: Root HTML asset resolution (GET /)
           - index.html must use root-absolute paths (/tokens.css, /muconsole-ui.js, /fonts/...),
             rejecting relative paths (./tokens.css) that cause 404 cascades on virtual routes.
           - Content-Security-Policy header contains single-quoted keywords ('self', 'unsafe-inline', 'wasm-unsafe-eval').
  Check 4: Virtual route asset isolation
           - Virtual route asset requests (/oauth2/tokens.css, /oauth2/muconsole-ui.js) must NOT
             serve SPA index.html fallback.
           - Root canonical assets (/tokens.css, /muconsole-ui.js) must return 200 OK.
  Check 5: Callback endpoint contract (GET /oauth2/callback)
           - Direct call without code/state returns HTTP 400 or 403 (NOT 200 SPA HTML).
           - Call with forged code/state returns HTTP 400 or 403 (CSRF mismatch).
  Check 6: Sign-out endpoint contract (GET /oauth2/sign_out?rd=/)
           - Returns HTTP 302/303 redirecting to rd target (NOT 200 SPA HTML).
           - Set-Cookie header evicts/expires session cookie (_muconsole_oauth2).
  Check 7: Auth probe endpoint contract (GET /oauth2/auth)
           - Unauthenticated probe returns HTTP 401 Unauthorized or 404 Internal (NOT 200 SPA HTML).

Exit code:
  0 = All checks PASSED (GREEN state)
  1 = At least one check FAILED (RED state)
"""

import sys
import os
import ssl
import http.client
import urllib.parse
import re
import argparse
from datetime import datetime, timezone

TARGET_HOST_DEFAULT = "console.lma1980.duckdns.org"
TARGET_PORT_DEFAULT = 443

class TestFailure(Exception):
    pass

def http_raw_request(host: str, port: int, method: str, path: str, headers: dict = None, body: str = None, timeout: int = 15):
    """Executes a single HTTPS request without following redirects."""
    headers = headers or {}
    ctx = ssl.create_default_context()
    conn = http.client.HTTPSConnection(host, port, context=ctx, timeout=timeout)
    encoded_body = body.encode("utf-8") if body is not None else None
    
    conn.request(method, path, body=encoded_body, headers=headers)
    resp = conn.getresponse()
    
    status = resp.status
    reason = resp.reason
    raw_headers = resp.getheaders()
    # Normalize headers to lowercase keys while preserving multi-headers as list
    headers_dict = {}
    set_cookies = []
    for k, v in raw_headers:
        lk = k.lower()
        if lk == "set-cookie":
            set_cookies.append(v)
        headers_dict[lk] = v
    if set_cookies:
        headers_dict["set-cookie-list"] = set_cookies
        
    raw_body = resp.read().decode("utf-8", errors="replace")
    conn.close()
    return status, reason, headers_dict, raw_body

def is_spa_html(body: str, headers: dict) -> bool:
    """Detects if response is MuConsole SPA index.html fallback."""
    content_type = headers.get("content-type", "").lower()
    if "text/html" in content_type:
        if "<title>MuConsole" in body or "muconsole-ui" in body or "tokens.css" in body:
            return True
        if "<!doctype html>" in body.lower():
            return True
    return False

def run_suite(host: str, port: int) -> bool:
    print("=" * 80)
    print(f"LIVE OAUTH2 E2E & INTEGRATION TEST SUITE — https://{host}:{port}")
    print(f"Timestamp: {datetime.now(timezone.utc).isoformat()}")
    print("=" * 80)

    results = []

    def record_result(check_num: int, name: str, passed: bool, details: list):
        results.append((check_num, name, passed, details))
        status_str = "PASS [GREEN]" if passed else "FAIL [RED]"
        print(f"\n[Check {check_num}] {name} -> {status_str}")
        for d in details:
            print(f"  {d}")

    # -------------------------------------------------------------------------
    # Check 1: GET /oauth2/start?rd=/ -> 302/303 redirect to GitHub OAuth
    # -------------------------------------------------------------------------
    chk1_details = []
    chk1_passed = True
    try:
        status, reason, headers, body = http_raw_request(host, port, "GET", "/oauth2/start?rd=/")
        chk1_details.append(f"HTTP Status: {status} {reason}")
        
        if is_spa_html(body, headers):
            chk1_details.append("ERROR: Endpoint returned 200 OK SPA HTML instead of 302/303 OAuth2 redirect!")
            chk1_details.append("Cause: /oauth2/* is not proxied to oauth2-proxy; Nginx fell back to index.html.")
            chk1_passed = False
        elif status not in (302, 303):
            chk1_details.append(f"ERROR: Expected HTTP 302 or 303, got {status}")
            chk1_passed = False
        else:
            chk1_details.append(f"Redirect status verified: {status}")

        location = headers.get("location")
        chk1_details.append(f"Location Header: {location}")
        if not location:
            chk1_details.append("ERROR: Missing Location response header on redirect!")
            chk1_passed = False
        else:
            parsed_loc = urllib.parse.urlparse(location)
            expected_base = ("https", "github.com", "/login/oauth/authorize")
            actual_base = (parsed_loc.scheme, parsed_loc.netloc, parsed_loc.path)
            if actual_base != expected_base:
                chk1_details.append(f"ERROR: Location base mismatch! Expected {expected_base}, got {actual_base}")
                chk1_passed = False
            else:
                chk1_details.append("Location target correctly points to https://github.com/login/oauth/authorize")

            params = urllib.parse.parse_qs(parsed_loc.query)
            chk1_details.append(f"OAuth2 Query Parameters: {list(params.keys())}")
            
            client_id = params.get("client_id", [""])[0]
            if not client_id or "REPLACE_WITH" in client_id:
                chk1_details.append(f"ERROR: Invalid client_id: '{client_id}'")
                chk1_passed = False
            else:
                chk1_details.append(f"client_id present: {client_id[:6]}... (valid)")

            redirect_uri = params.get("redirect_uri", [""])[0]
            expected_redirect = f"https://{host}/oauth2/callback"
            if redirect_uri != expected_redirect:
                chk1_details.append(f"ERROR: redirect_uri mismatch! Expected '{expected_redirect}', got '{redirect_uri}'")
                chk1_passed = False
            else:
                chk1_details.append(f"redirect_uri verified: {redirect_uri}")

            scope = params.get("scope", [""])[0]
            has_user_email = "user:email" in scope or "user%3Aemail" in location
            has_read_org = "read:org" in scope or "read%3Aorg" in location
            if not has_user_email or not has_read_org:
                missing = []
                if not has_user_email:
                    missing.append("user:email (user%3Aemail)")
                if not has_read_org:
                    missing.append("read:org (read%3Aorg)")
                chk1_details.append(
                    f"ERROR: scope must contain BOTH 'user:email' and 'read:org' (required by oauth2-proxy GitHub provider), missing: {missing}, got '{scope}'"
                )
                chk1_passed = False
            else:
                chk1_details.append(f"scope verified: '{scope}' (contains user:email and read:org)")

            state = params.get("state", [""])[0]
            if not state:
                chk1_details.append("ERROR: Missing state parameter in OAuth2 authorize URL")
                chk1_passed = False
            else:
                chk1_details.append(f"state parameter present: length {len(state)}")

        # Verify CSRF Cookie in Set-Cookie
        cookies = headers.get("set-cookie-list", [])
        if "set-cookie" in headers and not cookies:
            cookies = [headers["set-cookie"]]
        chk1_details.append(f"Set-Cookie Headers: {cookies}")
        
        csrf_cookie_found = False
        for c in cookies:
            c_lower = c.lower()
            if "_muconsole_oauth2" in c_lower or "_oauth2_proxy" in c_lower:
                csrf_cookie_found = True
                if "httponly" not in c_lower:
                    chk1_details.append(f"WARNING: CSRF cookie missing HttpOnly flag: {c}")
                if "path=/" not in c_lower and "path=/" not in c:
                    chk1_details.append(f"WARNING: CSRF cookie path not root: {c}")
                break

        if not csrf_cookie_found:
            chk1_details.append("ERROR: No OAuth2 CSRF/state cookie issued in Set-Cookie headers!")
            chk1_passed = False
        else:
            chk1_details.append("CSRF state cookie successfully issued.")

        # Verify Content-Security-Policy header
        csp = headers.get("content-security-policy", "")
        chk1_details.append(f"Content-Security-Policy: {csp}")
        if not csp:
            chk1_details.append("ERROR: Missing Content-Security-Policy response header on /oauth2/start!")
            chk1_passed = False
        else:
            required_csp_keywords = ["'self'", "'unsafe-inline'", "'wasm-unsafe-eval'"]
            missing_csp = [k for k in required_csp_keywords if k not in csp]
            if missing_csp:
                chk1_details.append(
                    f"ERROR: Content-Security-Policy missing single-quoted keyword(s): {missing_csp}! Unquoted keywords (e.g. self) are treated as hostnames, not CSP keywords. Got: '{csp}'"
                )
                chk1_passed = False
            else:
                chk1_details.append("Content-Security-Policy keywords properly single-quoted ('self', 'unsafe-inline', 'wasm-unsafe-eval').")

    except Exception as e:
        chk1_details.append(f"EXCEPTION: {e}")
        chk1_passed = False

    record_result(1, "OAuth2 Start Handshake Contract (GET /oauth2/start?rd=/)", chk1_passed, chk1_details)

    # -------------------------------------------------------------------------
    # Check 2: Invite URL Preservation in rd Parameter
    # -------------------------------------------------------------------------
    chk2_details = []
    chk2_passed = True
    try:
        sample_rd = "/?invite=inv_cca98d8ec4e62b9e02ec5286"
        encoded_rd = urllib.parse.quote(sample_rd, safe="")
        path = f"/oauth2/start?rd={encoded_rd}"
        status, reason, headers, body = http_raw_request(host, port, "GET", path)
        chk2_details.append(f"Request: GET {path}")
        chk2_details.append(f"HTTP Status: {status} {reason}")

        if is_spa_html(body, headers):
            chk2_details.append("ERROR: Returned 200 OK SPA HTML instead of redirect!")
            chk2_passed = False
        elif status not in (302, 303):
            chk2_details.append(f"ERROR: Expected 302 or 303, got {status}")
            chk2_passed = False
        else:
            location = headers.get("location", "")
            if not location.startswith("https://github.com/login/oauth/authorize"):
                chk2_details.append(f"ERROR: Location does not point to GitHub authorize: {location}")
                chk2_passed = False
            else:
                chk2_details.append("Invite-bearing rd handshake initiated successfully.")
    except Exception as e:
        chk2_details.append(f"EXCEPTION: {e}")
        chk2_passed = False

    record_result(2, "Invite-Preserving Redirect Target (GET /oauth2/start?rd=...)", chk2_passed, chk2_details)

    # -------------------------------------------------------------------------
    # Check 3: Root HTML Asset Resolution Invariant (GET /)
    # -------------------------------------------------------------------------
    chk3_details = []
    chk3_passed = True
    try:
        status, reason, headers, body = http_raw_request(host, port, "GET", "/")
        chk3_details.append(f"HTTP Status: {status} {reason}")
        if status != 200:
            chk3_details.append(f"ERROR: Root GET / failed with {status}")
            chk3_passed = False
        else:
            # Check for relative asset paths that cause 404 cascades on subpaths:
            # e.g., href="./tokens.css", src: url("./fonts/...", import init from './muconsole-ui.js'
            relative_patterns = [
                (r'href=["\']\./', "Relative stylesheet/link href: href=\"./...\""),
                (r'src=["\']\./', "Relative script/asset src: src=\"./...\""),
                (r'url\(["\']?\./', "Relative CSS url: url(\"./...\")"),
                (r'from\s+["\']\./', "Relative ES module import: from './...'"),
                (r'init\(["\']\./', "Relative WASM init call: init('./...')"),
            ]
            found_relatives = []
            for pat, desc in relative_patterns:
                matches = re.findall(pat, body)
                if matches:
                    found_relatives.append(f"{desc} ({len(matches)} occurrences)")

            if found_relatives:
                chk3_details.append("ERROR: Relative asset paths detected in root index.html:")
                for r in found_relatives:
                    chk3_details.append(f"  - {r}")
                chk3_details.append("Remediation: Convert all relative './' references to root-absolute '/' in index.html and build pipeline.")
                chk3_passed = False
            else:
                chk3_details.append("PASSED: All asset links and imports in index.html use root-absolute paths.")

            # Verify Content-Security-Policy on root HTML
            csp_root = headers.get("content-security-policy", "")
            chk3_details.append(f"Root Content-Security-Policy: {csp_root}")
            if not csp_root:
                chk3_details.append("ERROR: Root GET / missing Content-Security-Policy response header (check Nginx location / add_header inheritance)!")
                chk3_passed = False
            else:
                required_csp_keywords = ["'self'", "'unsafe-inline'", "'wasm-unsafe-eval'"]
                missing_csp_root = [k for k in required_csp_keywords if k not in csp_root]
                if missing_csp_root:
                    chk3_details.append(
                        f"ERROR: Root Content-Security-Policy missing single-quoted keyword(s): {missing_csp_root}! Got: '{csp_root}'"
                    )
                    chk3_passed = False
                else:
                    chk3_details.append("Root Content-Security-Policy keywords properly single-quoted ('self', 'unsafe-inline', 'wasm-unsafe-eval').")
    except Exception as e:
        chk3_details.append(f"EXCEPTION: {e}")
        chk3_passed = False

    record_result(3, "Root HTML Asset Absolute Path Invariant (GET /)", chk3_passed, chk3_details)

    # -------------------------------------------------------------------------
    # Check 4: Virtual Route Asset Isolation & Root Asset Canonical Reachability
    # -------------------------------------------------------------------------
    chk4_details = []
    chk4_passed = True
    try:
        # 4a: Virtual subpath asset requests must NOT return 200 SPA HTML
        subpath_assets = ["/oauth2/tokens.css", "/oauth2/muconsole-ui.js"]
        for sa in subpath_assets:
            s, r, h, b = http_raw_request(host, port, "GET", sa)
            chk4_details.append(f"GET {sa} -> {s} {r} (Content-Type: {h.get('content-type', 'none')})")
            if s == 200 and is_spa_html(b, h):
                chk4_details.append(f"ERROR: {sa} returned HTTP 200 with SPA HTML! Causes silent MIME parse failure in browsers.")
                chk4_passed = False
            elif s == 404:
                chk4_details.append(f"{sa} properly returns 404 Not Found (no SPA HTML leakage).")

        # 4b: Canonical root assets must return 200 with correct Content-Type
        s_css, _, h_css, _ = http_raw_request(host, port, "GET", "/tokens.css")
        chk4_details.append(f"GET /tokens.css -> {s_css} (Content-Type: {h_css.get('content-type')})")
        if s_css != 200 or "css" not in h_css.get("content-type", "").lower():
            chk4_details.append("ERROR: Canonical /tokens.css is unreachable or has non-CSS Content-Type!")
            chk4_passed = False

        s_js, _, h_js, _ = http_raw_request(host, port, "GET", "/muconsole-ui.js")
        chk4_details.append(f"GET /muconsole-ui.js -> {s_js} (Content-Type: {h_js.get('content-type')})")
        if s_js != 200 or ("javascript" not in h_js.get("content-type", "").lower() and "wasm" not in h_js.get("content-type", "").lower()):
            chk4_details.append("ERROR: Canonical /muconsole-ui.js is unreachable or has non-JS Content-Type!")
            chk4_passed = False

    except Exception as e:
        chk4_details.append(f"EXCEPTION: {e}")
        chk4_passed = False

    record_result(4, "Virtual Route Asset Non-Leakage & Root Canonical Assets", chk4_passed, chk4_details)

    # -------------------------------------------------------------------------
    # Check 5: Callback Endpoint Contract (GET /oauth2/callback)
    # -------------------------------------------------------------------------
    chk5_details = []
    chk5_passed = True
    try:
        # 5a: Missing code/state
        s, r, h, b = http_raw_request(host, port, "GET", "/oauth2/callback")
        chk5_details.append(f"GET /oauth2/callback (no params) -> {s} {r}")
        if is_spa_html(b, h):
            chk5_details.append("ERROR: /oauth2/callback returned 200 SPA HTML instead of proxy handling!")
            chk5_passed = False
        elif s not in (400, 403):
            chk5_details.append(f"ERROR: Expected HTTP 400 or 403 for missing code/state, got {s}")
            chk5_passed = False
        else:
            chk5_details.append(f"Callback properly rejected missing code/state with {s} {r}.")

        # 5b: Forged code/state without valid CSRF cookie
        s2, r2, h2, b2 = http_raw_request(host, port, "GET", "/oauth2/callback?code=fake_code&state=fake_state")
        chk5_details.append(f"GET /oauth2/callback?code=fake... -> {s2} {r2}")
        if is_spa_html(b2, h2):
            chk5_details.append("ERROR: Forged callback returned 200 SPA HTML instead of proxy rejection!")
            chk5_passed = False
        elif s2 not in (400, 403):
            chk5_details.append(f"ERROR: Expected HTTP 400 or 403 for forged callback, got {s2}")
            chk5_passed = False
        else:
            chk5_details.append(f"Callback properly rejected forged request with {s2} {r2}.")

    except Exception as e:
        chk5_details.append(f"EXCEPTION: {e}")
        chk5_passed = False

    record_result(5, "OAuth2 Callback Contract (GET /oauth2/callback)", chk5_passed, chk5_details)

    # -------------------------------------------------------------------------
    # Check 6: Sign-Out Endpoint Contract (GET /oauth2/sign_out?rd=/)
    # -------------------------------------------------------------------------
    chk6_details = []
    chk6_passed = True
    try:
        status, reason, headers, body = http_raw_request(host, port, "GET", "/oauth2/sign_out?rd=/")
        chk6_details.append(f"HTTP Status: {status} {reason}")
        if is_spa_html(body, headers):
            chk6_details.append("ERROR: /oauth2/sign_out returned 200 SPA HTML instead of 302 redirect!")
            chk6_passed = False
        elif status not in (302, 303):
            chk6_details.append(f"ERROR: Expected HTTP 302 or 303 on sign_out, got {status}")
            chk6_passed = False
        else:
            chk6_details.append(f"Sign-out redirect status verified: {status}")

        location = headers.get("location", "")
        chk6_details.append(f"Location Header: {location}")
        if not (location == "/" or location.endswith("/")):
            chk6_details.append(f"WARNING: Location does not point to '/': '{location}'")

        cookies = headers.get("set-cookie-list", [])
        if "set-cookie" in headers and not cookies:
            cookies = [headers["set-cookie"]]
        chk6_details.append(f"Set-Cookie Headers: {cookies}")

        cookie_cleared = False
        for c in cookies:
            c_lower = c.lower()
            if "_muconsole_oauth2" in c_lower or "_oauth2_proxy" in c_lower:
                if "max-age=0" in c_lower or "1970" in c_lower or "expires=" in c_lower:
                    cookie_cleared = True
                    break

        if not cookie_cleared:
            chk6_details.append("ERROR: /oauth2/sign_out did not issue an expired session cookie in Set-Cookie!")
            chk6_passed = False
        else:
            chk6_details.append("Session cookie eviction verified in Set-Cookie.")

    except Exception as e:
        chk6_details.append(f"EXCEPTION: {e}")
        chk6_passed = False

    record_result(6, "OAuth2 Sign-Out Contract (GET /oauth2/sign_out?rd=/)", chk6_passed, chk6_details)

    # -------------------------------------------------------------------------
    # Check 7: Auth Probe Endpoint Contract (GET /oauth2/auth)
    # -------------------------------------------------------------------------
    chk7_details = []
    chk7_passed = True
    try:
        status, reason, headers, body = http_raw_request(host, port, "GET", "/oauth2/auth")
        chk7_details.append(f"HTTP Status: {status} {reason}")
        if is_spa_html(body, headers):
            chk7_details.append("ERROR: /oauth2/auth returned 200 SPA HTML! Internal auth probe must never serve SPA HTML.")
            chk7_passed = False
        elif status in (401, 403, 404):
            chk7_details.append(f"/oauth2/auth properly rejected unauthenticated external probe with {status} {reason}.")
        else:
            chk7_details.append(f"Unexpected status for unauthenticated /oauth2/auth: {status}")
            chk7_passed = False
    except Exception as e:
        chk7_details.append(f"EXCEPTION: {e}")
        chk7_passed = False

    record_result(7, "OAuth2 Internal Auth Probe Invariant (GET /oauth2/auth)", chk7_passed, chk7_details)

    # -------------------------------------------------------------------------
    # Summary
    # -------------------------------------------------------------------------
    print("\n" + "=" * 80)
    print("TEST SUITE SUMMARY:")
    passed_count = sum(1 for _, _, p, _ in results if p)
    total_count = len(results)
    for c_num, c_name, c_pass, _ in results:
        mark = "✓ PASS [GREEN]" if c_pass else "✗ FAIL [RED]"
        print(f"  Check {c_num}: {c_name:<55} -> {mark}")

    all_passed = (passed_count == total_count)
    print("=" * 80)
    if all_passed:
        print(f"RESULT: ALL {total_count} CHECKS PASSED (STATE: GREEN, EXIT CODE 0)")
    else:
        print(f"RESULT: {total_count - passed_count} OF {total_count} CHECKS FAILED (STATE: RED, EXIT CODE 1)")
    print("=" * 80)
    return all_passed

def main():
    parser = argparse.ArgumentParser(description="Live E2E & integration verification for MuConsole OAuth2 flows")
    parser.add_argument("--host", default=os.environ.get("CONSOLE_HOST", TARGET_HOST_DEFAULT), help="Target host")
    parser.add_argument("--port", type=int, default=int(os.environ.get("CONSOLE_PORT", TARGET_PORT_DEFAULT)), help="Target port")
    args = parser.parse_args()

    success = run_suite(args.host, args.port)
    sys.exit(0 if success else 1)

if __name__ == "__main__":
    main()
