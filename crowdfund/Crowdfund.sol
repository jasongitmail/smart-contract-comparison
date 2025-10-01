// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

/**
 * @title Crowdfund
 * @dev Time-based crowdfunding with automatic refunds on failure
 */
contract Crowdfund {
    address public immutable owner;
    uint256 public immutable goal;
    uint256 public immutable deadline;
    uint256 public totalRaised;
    bool public finalized;

    mapping(address => uint256) public contributions;

    event Contributed(address indexed contributor, uint256 amount);
    event GoalReached(uint256 totalRaised);
    event Refunded(address indexed contributor, uint256 amount);
    event Withdrawn(address indexed owner, uint256 amount);

    constructor(uint256 _goal, uint256 _durationBlocks) {
        require(_goal > 0, "Goal must be greater than zero");
        require(_durationBlocks > 0, "Duration must be greater than zero");

        owner = msg.sender;
        goal = _goal;
        deadline = block.number + _durationBlocks;
    }

    /**
     * @dev Contribute funds to the campaign
     */
    function contribute() external payable {
        require(block.number < deadline, "Campaign has ended");
        require(!finalized, "Campaign already finalized");
        require(msg.value > 0, "Must contribute a positive amount");

        contributions[msg.sender] += msg.value;
        totalRaised += msg.value;

        emit Contributed(msg.sender, msg.value);

        if (totalRaised >= goal) {
            emit GoalReached(totalRaised);
        }
    }

    /**
     * @dev Check if campaign was successful
     */
    function isSuccessful() public view returns (bool) {
        return totalRaised >= goal;
    }

    /**
     * @dev Withdraw funds if goal was reached (owner only)
     */
    function withdraw() external {
        require(msg.sender == owner, "Only owner can withdraw");
        require(block.number >= deadline, "Campaign still active");
        require(!finalized, "Already finalized");
        require(isSuccessful(), "Goal not reached");

        finalized = true;
        uint256 amount = totalRaised;

        (bool success, ) = owner.call{value: amount}("");
        require(success, "Transfer failed");

        emit Withdrawn(owner, amount);
    }

    /**
     * @dev Claim refund if goal was not reached
     */
    function refund() external {
        require(block.number >= deadline, "Campaign still active");
        require(!finalized, "Already finalized");
        require(!isSuccessful(), "Goal was reached, no refunds");

        uint256 amount = contributions[msg.sender];
        require(amount > 0, "No contribution to refund");

        contributions[msg.sender] = 0;

        (bool success, ) = msg.sender.call{value: amount}("");
        require(success, "Refund failed");

        emit Refunded(msg.sender, amount);
    }

    /**
     * @dev Finalize failed campaign (allows batch processing)
     */
    function finalize() external {
        require(block.number >= deadline, "Campaign still active");
        require(!finalized, "Already finalized");
        require(!isSuccessful(), "Goal was reached");

        finalized = true;
    }
}
