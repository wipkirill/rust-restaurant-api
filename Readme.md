# Restaurant API in Rust(first project)

## Requirements
The restaurant staff, using their devices, should be able to do three things: add one or more items to a table with a number, remove an item from a table, and check what items are still on a table.

When a new request is made, the system must store the item, table number, and how long it takes to cook.

If a request is made to delete something, the system should remove the specified item from the specified table.

If a request is made to check, the system should display all items for a given table number, or a specific item for a specific table number.

The system should be able to handle at least 10 requests happening at the same time.

The staff can choose from a set of at least 100 tables when making their requests.

The system can randomly decide the time it takes to prepare an item, anywhere between 5 to 15 minutes.

The time it takes to prepare an item doesn't have to be tracked in real-time; it only needs to be set when the item is created and removed when the item is deleted.

## Build and run
Use ```cargo build --release``` for a prod build. The ```restaurant-api``` app itself accepts the following command line arguments:
```
Restaurant API

Usage: restaurant-api [OPTIONS]

Options:
  -a, --address <ADDRESS>          Server address [default: 127.0.0.1]
  -p, --port <PORT>                Server port 0-65535 [default: 3000]
  -n, --num-clients <NUM_CLIENTS>  [default: 10]
  -h, --help                       Print help
```
Specify the number of clients with the ``num-clients`` option. Set 0 to run without spinning clients and explore the API in Postman (file in repo root).

## Data structures and storage choice
Explore the src/domain folder to find business objects and their fields. I based them on tuple structs types and try_from properties for easy validation. We can therefore claim that any instance of ItemId, TableId etc will satisfy all our validation constraints.

### Storage
There are two options: in-memory and sqlite(default). Repository interface has been implemented for both structs (see repository folder). I used mutexes to protect the internal datastructure or connection object in a multi-threaded environment. However, this introduces locking and may reduce performance. For the in-memory case, an RwLock could partially solve the problem as multiple read requests are served without locking. For the sqlite case, the chosen library didn't support connection pooling, and here we might think of replacing it. Connection pooling would share and reuse multiple instances of connections to provide multi-threaded access without locking (until the pool is emptied).

### Versioning mechanism and soft deletes
I have added a new field called ``version`` to the Item struct. My main idea is to use it for the case when multiple readers get the latest version of an item and then try to write changes to the DB in parallel. The writer who updates the DB first will hit the version increment and other writers will fail because they are using an older version than in the DB. They will need to update the item by reading it again.

Soft deletion is used. The field ```deleted=1`` marks an item as deleted. The DB schema enforces that there can only be one non-deleted item and multiple deleted items with the same table_id and item_id. A unique index is created to support this.

## Improvement and scaling considerations
- Make requests per second, DAU assumptions, peak usage.
- Hiding application instances behind a load balancer. A typical server can handle around 10K rps.
- Introduce a rate limiter when needed
- Identify bottlenecks. If DB is a major one, check data access patterns: i.e. more reads or writes? 
    1. Data locality. Make a DB sharding by e.g. table_id. All elements of a with same table_id end up in one DB instance.
    A vertical or horizontal partitioning can be applied.
    2. Use a memcache to speed up reads. Invalidate cache when item gets updated.


## References and credits
1. Official Rust docs
2. https://dev.to/deciduously/oops-i-did-it-againi-made-a-rust-web-api-and-it-was-not-that-difficult-3kk8
3. https://www.lpalmieri.com/posts/2020-08-09-zero-to-production-3-how-to-bootstrap-a-new-rust-web-api-from-scratch/
4. https://alexis-lozano.com/hexagonal-architecture-in-rust-1/
 