// SPDX-License-Identifier: MIT
pragma solidity ^0.8.17;

import "./Collection.sol";
import "./CollectionRegistryContext.sol";

contract CollectionAggregator is CollectionRegistryContext {
    event CollectionCreated(address indexed collection, string name, string symbol);
    event TokenMinted(address indexed collection, address recipient, uint256 tokenId, string tokenUri);

    constructor() {}

    receive() external payable {
        // ...
    }

    fallback() external payable {
        // ...
    }

    function createCollection(string memory name, string memory symbol) public {
        Collection collection = new Collection(address(this), name, symbol);
        address collection_ = address(collection);
        collectionRegistrySet(collection_);
        emit CollectionCreated(collection_, name, symbol);
        return;
    }

    function mint(address payable collection, address recipient, string memory tokenUri) public {
        require(recipient != address(0), "ERC721: address zero is not a valid recipient.");
        require(collectionRegistryIsExist(collection), "Collection does not exist.");
        Collection collection_ = Collection(collection);
        uint256 tokenId = collection_.mint(recipient, tokenUri);
        emit TokenMinted(collection, recipient, tokenId, tokenUri);
        return;
    }
}