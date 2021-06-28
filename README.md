# kademlia-dht

Implementation of the Kademlia DHT protocol in Rust

## Lib structure

```
src/
	key.res		---> Implementation of the 256bits unique ID
	node.rs		---> Node struct definition
	network.rs	---> Network module used to issue RPCs
	routing.rs	---> Routing Table implementation using vectors
	protocol.rs ---> Main library API
	utils.rs	---> General utilities functions
	main.rs		---> Example program
	lib.rs		---> Main lib file
```

## Implemented features

Features specified in the paper that are implemented in this lib

-   [x] Keys
-   [x] XOR Distance between Keys
-   [x] KBuckets

    -   represented as a `Vec` of `Vec`s. A max of 256 kbuckets is set, each of them containing up to 20 elements

-   [x] PING
-   [x] STORE
-   [x] FIND_NODE
-   [x] FIND_VALUE
-   [x] Node lookup

## Missing features

-   [ ] Republishing of `<key, value>` pairs every hour

    -   [ ] original publisher republishes ever 24 hours

-   [ ] expiration date on `<key, value>` pairs

-   [ ] replicate closest `<key, value>` pairs when a node joins the network

-   [ ] if no lookup has been performed for an hour in a `kbucket`, that bucket must be refreshed

## Enhancements

-   [ ] better nodes lookup algorithm, as described in the paper

## References

-   Kademlia: A Peer-to-peer Information System
    Based on the XOR Metric by Petar Maymounkov and David Mazières (original paper)

-   Implementation of the Kademlia Distributed Hash Table by Bruno Spori

-   Kademlia: A Design Specification by XLattice project

-   TinyTorrent: Implementing a Kademlia Based DHT
    for File Sharing by Sierra Kaplan-Nelson, Jestin Ma, Jake Rachleff
