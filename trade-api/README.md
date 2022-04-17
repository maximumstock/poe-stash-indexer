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
    "buy": "Exalted Orb"
}
```

```json
HTTP/1.1 200 OK
Server: nginx/1.20.1
Date: Thu, 24 Feb 2022 22:46:25 GMT
Content-Type: application/json
Transfer-Encoding: chunked
Connection: close
Content-Encoding: gzip

{
  "count": 10,
  "offers": [
    {
      "item_id": "75011829a746b44c7620a54f072ceceb1ee76b48c2aea593d6ce6aa621307203",
      "stash_id": "8dc7f7443f831cd8424dc1ad1352022d8262b580b140d629c62d2b375f65c973",
      "sell": "Chaos Orb",
      "buy": "Exalted Orb",
      "seller_account": "farmigod",
      "stock": 23,
      "conversion_rate": 0.007936508,
      "created_at": 1645576637
    },
    {
      "item_id": "6f2d9a7ddd47ff569bbfe3261ec6956b76b33f4b8d3c259985638f958ea12210",
      "stash_id": "a82e68371c16f76d11c4033f65314e48ee14f31b1f75b9b164c1cd1d6320fd26",
      "sell": "Chaos Orb",
      "buy": "Exalted Orb",
      "seller_account": "dawyd9",
      "stock": 560,
      "conversion_rate": 0.008,
      "created_at": 1645576644
    },
    {
      "item_id": "b2d76468e00114737f720c335244b2a5782f142bd41b07b87235589b349f9ce1",
      "stash_id": "14b57e42d7e8173681cdf7b7d364927a944c662a689df8981c52f5f181787503",
      "sell": "Chaos Orb",
      "buy": "Exalted Orb",
      "seller_account": "Str3Rx",
      "stock": 1051,
      "conversion_rate": 0.008,
      "created_at": 1645576632
    },
    {
      "item_id": "a05642ae31af0b6a6fc40bffc149cd7a8134f5374ab210d44382d7000e10b0cc",
      "stash_id": "4589ca34a5284546485e859a549b8b8ac385786737428cf472351fe4b48ac440",
      "sell": "Chaos Orb",
      "buy": "Exalted Orb",
      "seller_account": "Zarebrant",
      "stock": 5000,
      "conversion_rate": 0.011111111,
      "created_at": 1645576644
    },
    {
      "item_id": "69071e95db6e3ae65c7d86a60d159aa188275e22fd22e579215a5c7c0b5d0f5e",
      "stash_id": "3a0554e0a3a76388cf39dd1e0075649d871d670dfeeec495fa5859096172b200",
      "sell": "Chaos Orb",
      "buy": "Exalted Orb",
      "seller_account": "lcd900",
      "stock": 2174,
      "conversion_rate": 0.007874016,
      "created_at": 1645742778
    },
    {
      "item_id": "09e50764d9f5f9b4d06fcff3f3f5a1fa56d469c2646b338f74fb0f428b438ff2",
      "stash_id": "7a4f3f84594c713a985578fea2c52b841a6ebdfd3b7110b4aa61df984ee844df",
      "sell": "Chaos Orb",
      "buy": "Exalted Orb",
      "seller_account": "ohdombro",
      "stock": 723,
      "conversion_rate": 0.00952381,
      "created_at": 1645576643
    },
    {
      "item_id": "116c9a760055bc3263997e3ed60dce3855cdcfc6523e71b2670de8dafea95b2c",
      "stash_id": "a3cf22cc4b66add6a80b08a2ef600a7afb7556521ddca2b46a69da6425a0c7a7",
      "sell": "Chaos Orb",
      "buy": "Exalted Orb",
      "seller_account": "empty93",
      "stock": 2572,
      "conversion_rate": 0.007874016,
      "created_at": 1645576637
    },
    {
      "item_id": "a7f31658cb202c10c74a5d06cf3625d00b7c3e6b98a328b89c0d029218ebfefa",
      "stash_id": "af87dc364e17a17f7de4df4536403bceb16632cd6c171e4b4cf250385ebce365",
      "sell": "Chaos Orb",
      "buy": "Exalted Orb",
      "seller_account": "nihaohao",
      "stock": 2738,
      "conversion_rate": 0.007843138,
      "created_at": 1645576644
    },
    {
      "item_id": "e6c423d836cdd1ccd6decfa1841f38b59ad65b0fb87875971542b7db062046da",
      "stash_id": "8c5ecb5d8626c25b2aba010d48b5bb7b61ba8aa95ab54a2354b3565df953b71d",
      "sell": "Chaos Orb",
      "buy": "Exalted Orb",
      "seller_account": "zomsie",
      "stock": 2074,
      "conversion_rate": 0.007751938,
      "created_at": 1645742775
    },
    {
      "item_id": "7565a3dd193b01ed84a135ab6f9c62b82837396d072613df74355e54f57a6fa2",
      "stash_id": "1f4449dad14bf23b1b08a9e02911f54d86c464b0a23e577624743f6f21908b08",
      "sell": "Chaos Orb",
      "buy": "Exalted Orb",
      "seller_account": "inwardheel8",
      "stock": 536,
      "conversion_rate": 0.012048192,
      "created_at": 1645576632
    }
  ]
}
```
