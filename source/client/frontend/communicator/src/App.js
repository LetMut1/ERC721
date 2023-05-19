import { useEffect, useState } from 'react';
import './App.css';
import collectionAggregatorContractJson from './contract_abi/CollectionAggregator.json';
import { ethers } from 'ethers';

// CHANGE THIS after deploying.
const contractAddress = "0xD24894f5b970Fa36BBbaae9402d92DABF1f2aC50";

function App() {
	const [currentAccount, setCurrentAccount] = useState(null);
	const [name_, setName] = useState('');
	const [symbol, setSymbol] = useState('');
	const [collectionAddress, setCollectionAddress] = useState('');
	const [recipient, setRecipient] = useState('');
	const [tokenUri, setTokenUri] = useState('');

	const checkWalletIsConnected = async () => {
		const { ethereum } = window;

		const accounts = await ethereum.request({ method: 'eth_accounts' });

		if (accounts.length !== 0) {
			const account = accounts[0];
			setCurrentAccount(account);
		} else {
			console.log("No authorized account found");
		}
  	}

  	const connectWalletHandler = async () => {
    	const { ethereum } = window;

		try {
			const accounts = await ethereum.request({ method: 'eth_requestAccounts' });

			setCurrentAccount(accounts[0]);
		} catch (error) {
			console.log(error)
		}
	}

	const createCollection = async () => {
		try {
			const { ethereum } = window;

			if (ethereum) {
				const provider = new ethers.providers.Web3Provider(ethereum);

				const signer = provider.getSigner();

				const collectionAggregatorContract = new ethers.Contract(contractAddress, collectionAggregatorContractJson.abi, signer);

				let result = await collectionAggregatorContract.createCollection(name_, symbol);

				await result.wait();

				var collectionQuantity = (await collectionAggregatorContract.collectionRegistryGetLength()).toNumber();

				var collectionAddress = (await collectionAggregatorContract.collectionRegistryGetByIndex(collectionQuantity - 1)).toString();

				setCollectionAddress(collectionAddress);
			} else {
				console.log("Ethereum object does not exist");
			}

			setName("");

			setSymbol("");

			window.alert("Success");
		} catch (error) {
			console.log(error);

			window.alert("Error. Check logs.");
		}
	}

	const mint = async () => {
		try {
			const { ethereum } = window;

			if (ethereum) {
				const provider = new ethers.providers.Web3Provider(ethereum);

				const signer = provider.getSigner();

				var collectionAddress_ = ethers.utils.getAddress(collectionAddress);

				var recipient_ = ethers.utils.getAddress(recipient);

				const collectionAggregatorContract = new ethers.Contract(contractAddress, collectionAggregatorContractJson.abi, signer);

				let result = await collectionAggregatorContract.mint(collectionAddress_, recipient_, tokenUri);

				await result.wait();
			} else {
				console.log("Ethereum object does not exist");
			}

			setRecipient("");

			setTokenUri("");

			window.alert("Success");
		} catch (error) {
			console.log(error);

			window.alert("Error. Check logs.");
		}
	}

	const onChangeName = (event) => {
		setName(event.target.value);
	};

	const onChangeSymbol = (event) => {
		setSymbol(event.target.value);
	};

	const onChangeCollectionAddress = (event) => {
		setCollectionAddress(event.target.value);
	};

	const onChangeRecipient = (event) => {
		setRecipient(event.target.value);
	};

	const onChangeTokenUri = (event) => {
		setTokenUri(event.target.value);
	};

	const connectWallet = () => {
		return (
			<button onClick = {connectWalletHandler} className = 'cta-button connect-wallet-button'>Connect wallet</button>
		)
	}

	const getInterface = () => {
		return (
			<div>
				<label htmlFor = "name_">Name: </label>
				<input
					type = "text"
					id = "name_"
					name = "name_"
					onChange = {onChangeName}
					value = {name_}
				/>
				<br></br>
				<br></br>
				<label htmlFor = "symbol">Symbol: </label>
				<input
					type = "text"
					id = "symbol"
					name = "symbol"
					onChange = {onChangeSymbol}
					value = {symbol}
				/>
				<br></br>
				<br></br>
				<button onClick = {createCollection} className = 'cta-button contract-button'>Create NFT collection</button>
				<br></br>
				<br></br>
				<br></br>
				<br></br>
				<br></br>
				<br></br>
				<label htmlFor = "collectionAddress">Collection address: </label>
				<input
					type = "text"
					id = "collectionAddress"
					name = "collectionAddress"
					onChange = {onChangeCollectionAddress}
					value = {collectionAddress}
				/>
				<br></br>
				<br></br>
				<label htmlFor = "recipient">Recipient: </label>
				<input
					type = "text"
					id = "recipient"
					name = "recipient"
					onChange = {onChangeRecipient}
					value = {recipient}
				/>
				<br></br>
				<br></br>
				<label htmlFor = "tokenUri">Token URI: </label>
				<input
					type = "text"
					id = "tokenUri"
					name = "tokenUri"
					onChange = {onChangeTokenUri}
					value = {tokenUri}
				/>
				<br></br>
				<br></br>
				<button onClick = {mint} className = 'cta-button contract-button'>Mint</button>
			</div>
		)
	}

	useEffect(() => {
		checkWalletIsConnected();
	}, [])

	return (
		<div className = 'main-app'>
			<div>
				{currentAccount ? getInterface() : connectWallet()}
			</div>
		</div>
	)
}

export default App;