// SPDX-License-Identifier: MIT
pragma solidity ^0.8.17;

import "./../node_modules/@openzeppelin/contracts/token/ERC721/extensions/ERC721URIStorage.sol";
import "./../node_modules/@openzeppelin/contracts/utils/Counters.sol";
import "./CollectionAggregator.sol";

contract Collection is ERC721URIStorage {
    using Counters for Counters.Counter;

    Counters.Counter private counter_;
    address private methodCaller_;

    constructor(address methodCaller, string memory name, string memory symbol)
        ERC721(name, symbol)
    {
        methodCaller_ = methodCaller;
    }

    receive() external payable {
        // ...
    }

    fallback() external payable {
        // ...
    }

    function mint(address recipient, string memory tokenURI) public returns (uint256) {
        require(msg.sender == methodCaller_, "Bad call.");
        uint256 tokenId = counter_.current();
        _safeMint(recipient, tokenId);
        _setTokenURI(tokenId, tokenURI);
        counter_.increment();
        return tokenId;
    }

    function getMethodCaller() public view returns (address) {
        return methodCaller_;
    }
}