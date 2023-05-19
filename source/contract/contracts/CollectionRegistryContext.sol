// SPDX-License-Identifier: MIT
pragma solidity ^0.8.17;

import "./EnumerableAddressMap.sol";

abstract contract CollectionRegistryContext {
    using EnumerableAddressMap for EnumerableAddressMap.EnumerableAddressMap_;

    EnumerableAddressMap.EnumerableAddressMap_ private collectionRegistry_;

    constructor() {}

    function collectionRegistryGetByIndex(uint index) public view returns (address) {
        return collectionRegistry_.getByIndex(index);
    }

    function collectionRegistryGetLength() public view returns (uint) {
        return collectionRegistry_.getLength();
    }

    function collectionRegistryIsExist(address collection) public view returns (bool) {
        return collectionRegistry_.isExist(collection);
    }

    function collectionRegistrySet(address collection) internal {
        collectionRegistry_.set(collection);
        return;
    }
}