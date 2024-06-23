# ADDIX+FOMO validator node rewards distribution blueprint

## This blueprint has the following characteristics
* Users have to register anonymously with their Radix wallet to get an NFT. A UserEvent is emitted when this happens.
* The owner can deposit the rewards ahead of time (rug proof), before actually assigning them to users (different rewards allowed?).
* The presence of a scheduled offchain operation that takes a snapshot of the LSU distribution is required; this scheduled operation must inform the component about the rewards to be distributed.
* A DistributionEvent is emitted when the distribution happens.
* Each user can withdraw his rewards whenever he wants.
