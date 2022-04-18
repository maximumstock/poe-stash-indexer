# trade-api

This service exposes a REST-like API that lets you query these currency and bulk item offers.

## Endpoints

### `POST /trade`

Let's you search for currency & bulk item offers.

The following example queries for ten offers where players sell their `Chaos Orb`'s for `Exalted Orb`'s.
This endpoint always returns the N most recent offers that match your query.

Query Parameters:

- `limit` - default: 50, maximum: 200
- `league` - default: "Archnemesis", enum: ["Archnemesis", "Hardcore Archnemesis"]

```json
POST http://trade.maximumstock.net/trade?limit=10
content-type: application/json

{
    "sell": "Chaos Orb",
    "buy": "Exalted Orb",
    "limit": 3
}
```

```json
HTTP/1.1 200 OK
Server: nginx/1.20.1
Date: Mon, 18 Apr 2022 01:31:41 GMT
Content-Type: application/json
Transfer-Encoding: chunked
Connection: close
Content-Encoding: gzip

{
  "count": 5,
  "offers": [
    {
      // An auto-generated id from GGG that identifies an item within the context of the owning account.
      // This `item_id` changes as soon as the item moves to another account.
      "item_id": "cf99651d9c359ba1d05420a3add67e825e253abeda97357568bdce697346a27a",
      // A auto-generated id from GGG that globally identifies a stash across all accounts.
      "stash_id": "43233708689f72e19fd8bb008b9e075a4f1b76db20f7ec6052d120cfaa444531",
      // the item the seller sells
      "sell": "Chaos Orb",
      // the item the seller desires
      "buy": "Exalted Orb",
      // account name of the selling party
      "seller_account": "гуар",
      // number of units of [sell] that are available
      "stock": 1,
      // the number/fraction of units of [buy] you have to give the seller for one unit of [sell]
      "conversion_rate": 0.0058139535,
      // timestamp when offer was processed by the trade service
      "created_at": "2022-04-18T01:31:39"
    },
    {
      "item_id": "af25aa5bf4762c5e05d182de69b31f639f5a4a95dfa0be70bb4cf93bbe097694",
      "stash_id": "9fede136ec177bb8ae6658ab922bd8e71d7e343252abcbf4b04da06d0b58b49e",
      "sell": "Chaos Orb",
      "buy": "Exalted Orb",
      "seller_account": "luckyv90",
      "stock": 143,
      "conversion_rate": 0.00625,
      "created_at": "2022-04-18T01:31:20"
    },
    {
      "item_id": "d7de9e59c8364a5fc5583a8c5642f7115f9e68ec9379d5401af203b08f15cd34",
      "stash_id": "ed1f2f3701516e7814682ac87a79e91fc27a06ad8dd7a8be7498be42b01bc66d",
      "sell": "Chaos Orb",
      "buy": "Exalted Orb",
      "seller_account": "birdhunter88",
      "stock": 1162,
      "conversion_rate": 0.006666667,
      "created_at": "2022-04-18T01:31:06"
    },
    {
      "item_id": "b6a6ed9cff46a532426016e56a1ec3dcdae19b9b0e9f3588099f072dc04a43c4",
      "stash_id": "c4c6feef00fdb151c4efc619f6d7c6c03e6e2e73704a4bc2070c6f3d0b4c30be",
      "sell": "Chaos Orb",
      "buy": "Exalted Orb",
      "seller_account": "Cyranno2",
      "stock": 1834,
      "conversion_rate": 0.057692308,
      "created_at": "2022-04-18T01:31:02"
    },
    {
      "item_id": "b3ca167ddca3799c9ca56f6b06748f4c3c2468c5db8060cf94ad140f03c1b3c9",
      "stash_id": "2d84f76ab6f8ffbef5f59ea49f3c89d1b3e5ec08899780c0cfaf1e9ebdfb7a01",
      "sell": "Chaos Orb",
      "buy": "Exalted Orb",
      "seller_account": "Saejar",
      "stock": 1786,
      "conversion_rate": 0.006666667,
      "created_at": "2022-04-18T01:31:01"
    }
  ]
}
```
