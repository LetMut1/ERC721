// SPDX-License-Identifier: MIT
pragma solidity ^0.8.17;

library ExistableUint {
    struct ExistableUint_ {
        uint value_;
        bool isExist_;
    }

    function isExist(ExistableUint_ storage existableUint) internal view returns (bool) {
        return existableUint.isExist_ != getBoolDefaultValue();
    }

    function create(uint value) internal pure returns (ExistableUint_ memory) {
        return ExistableUint_({value_: value, isExist_: !getBoolDefaultValue()});
    }

    function getBoolDefaultValue() private pure returns (bool) {
        return false;
    }
}