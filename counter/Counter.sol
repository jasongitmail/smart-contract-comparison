// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

/**
 * @title Counter
 * @dev Simple increment/decrement counter with owner-only access control
 */
contract Counter {
    uint256 private count;
    address private immutable owner;

    event Incremented(uint256 newCount);
    event Decremented(uint256 newCount);

    constructor() {
        owner = msg.sender;
        count = 0;
    }

    modifier onlyOwner() {
        require(msg.sender == owner, "Only owner can modify counter");
        _;
    }

    /**
     * @dev Increment the counter by 1
     */
    function increment() external onlyOwner {
        count += 1;
        emit Incremented(count);
    }

    /**
     * @dev Decrement the counter by 1
     */
    function decrement() external onlyOwner {
        require(count > 0, "Counter cannot go below zero");
        count -= 1;
        emit Decremented(count);
    }

    /**
     * @dev Get the current count (view function - no gas for external calls)
     * @return The current counter value
     */
    function getCount() external view returns (uint256) {
        return count;
    }

    /**
     * @dev Get the owner address
     * @return The owner's address
     */
    function getOwner() external view returns (address) {
        return owner;
    }
}
