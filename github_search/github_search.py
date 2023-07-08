import json
import re
import requests
import time
from requests.adapters import HTTPAdapter
from requests.packages.urllib3.util.retry import Retry


def get_next(links):
    print(links)
    for link in links.split(','):
        m = re.match('<([^>]+)>; rel="([^"]+)"', link.strip())
        if not m:
            print('Failed to parse: %r' % (links,))
            raise ValueError
        if m.group(2) == 'next':
            return m.group(1)
    return None


def main():
    session = requests.Session()
    retry = Retry(
        total=3,
        read=3,
        connect=3,
        backoff_factor=10,
        status_forcelist=(403, 500, 502, 504),
    )
    adapter = HTTPAdapter(max_retries=retry)
    session.mount('http://', adapter)
    session.mount('https://', adapter)

    f_raw = open('search_raw', 'w')
    f_results = open('search_results', 'w')
    results = []
    auth = requests.auth.HTTPBasicAuth('ehuss', 'xxx')
    step = 50
    for size in range(0, 20000, step):
        srange = '%i..%i' % (size, size + step)
        url = 'https://api.github.com/search/code?q=filename%%3ACargo.toml+workspace+size:%s&type=Code&per_page=100' % (srange,)
        print('fetch: %s' % (url,))
        r = session.get(url, auth=auth)
        print(r)
        j = r.json()
        print('found %s' % (j['total_count'], ))
        if j['total_count'] > 1000:
            print('Too many for step %s: %s' % (srange, j['total_count']))
            raise ValueError
        if j.get('incomplete_results'):
            print('WARNING: incomplete')
        results.append(j)
        last_result = r
        num_this_range = len(results[-1]['items'])
        while 1:
            time.sleep(5)
            try:
                links = last_result.headers['Link']
            except KeyError:
                print('last_result %r had no link: %s' % (last_result, last_result.headers))
                break
            next_link = get_next(links)
            if not next_link:
                break
            print(next_link)
            r = session.get(next_link, auth=auth)
            print(r)
            results.append(r.json())
            num_this_range += len(results[-1]['items'])
            last_result = r
        if num_this_range >= 1000:
            print("ERROR: range %s had too many results" % (srange,))
    for result in results:
        for entry in result['items']:
            f_results.write('%s %s\n' % (entry['repository']['html_url'], entry['html_url']))
    json.dump(results, f_raw)


if __name__ == '__main__':
    main()
