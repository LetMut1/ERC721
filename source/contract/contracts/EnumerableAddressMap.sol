// SPDX-License-Identifier: MIT
pragma solidity ^0.8.17;

import "./ExistableUint.sol";

library EnumerableAddressMap {
    using ExistableUint for ExistableUint.ExistableUint_;

    struct EnumerableAddressMap_ {
        mapping (address => ExistableUint.ExistableUint_) addressRegistryResolver_;
        address[] addressRegistry_;
    }

    function set(EnumerableAddressMap_ storage enumerableAddressMap, address address_) internal {
        enumerableAddressMap.addressRegistryResolver_[address_] = ExistableUint.create(
            enumerableAddressMap.addressRegistry_.length
        );
        enumerableAddressMap.addressRegistry_.push(address_);
        return;
    }

    function getByIndex(EnumerableAddressMap_ storage enumerableAddressMap, uint index) internal view returns (address) {
        require(index < enumerableAddressMap.addressRegistry_.length, "Bad index.");
        return enumerableAddressMap.addressRegistry_[index];
    }

    function isExist(EnumerableAddressMap_ storage enumerableAddressMap, address address_) internal view returns (bool) {
        return enumerableAddressMap.addressRegistryResolver_[address_].isExist();
    }

    function getLength(EnumerableAddressMap_ storage enumerableAddressMap) internal view returns (uint) {
        return enumerableAddressMap.addressRegistry_.length;
    }
}