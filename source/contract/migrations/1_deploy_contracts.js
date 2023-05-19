const collectionAggregator = artifacts.require("CollectionAggregator");

module.exports = function(deployer) {
  deployer.deploy(collectionAggregator);
};
