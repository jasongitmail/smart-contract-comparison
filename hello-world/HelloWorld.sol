// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

/**
 * @title HelloWorld
 * @dev Store and retrieve a message with tracking of the last updater
 */
contract HelloWorld {
    string private message;
    address private lastUpdater;

    event MessageUpdated(string newMessage, address updater);

    constructor() {
        message = "Hello, World!";
        lastUpdater = msg.sender;
    }

    /**
     * @dev Set a new message
     * @param newMessage The message to store
     */
    function setMessage(string memory newMessage) public {
        require(bytes(newMessage).length > 0, "Message cannot be empty");
        require(bytes(newMessage).length <= 280, "Message too long (max 280 bytes)");
        message = newMessage;
        lastUpdater = msg.sender;
        emit MessageUpdated(newMessage, msg.sender);
    }

    /**
     * @dev Get the current message
     * @return The stored message
     */
    function getMessage() public view returns (string memory) {
        return message;
    }

    /**
     * @dev Get the address of who last updated the message
     * @return The address of the last updater
     */
    function getLastUpdater() public view returns (address) {
        return lastUpdater;
    }
}
