
The demonstration of work with Solidity smartcontracts. There is:
1. Solidity smartcontract for deploying a NFT collection (ERC721) with
arguments: name, symbol. Contract emits the `CollectionCreated(address collection, name, symbol)` and `TokenMinted(address collection, address recipient, tokenId, tokenUri)` events.
2. Simple HTTP-server with Redis storage to handle emitted events and
serve it by requests.
3. Simple frontend application that interacts with the smartcontract and has the `Create a new NFT collection with specified name and symbol (from user input)` and `Mints a new NFT with specified collection address (only created on 3.a), tokenId, tokenUri` functionality.
<br>
<br>
This is not production-ready code, because there is a lot of not handled "bad" moments.