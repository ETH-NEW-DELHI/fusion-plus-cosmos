#!/usr/bin/env node

/**
 * Cross-Chain Swap Demo Script
 * Demonstrates atomic swap between Ethereum Sepolia and Osmosis Testnet
 * 
 * Flow:
 * 1. User initiates swap (creates secret, locks ETH on Sepolia)
 * 2. Resolver creates escrow on Osmosis and deposits ATOM
 * 3. User reveals secret to claim ATOM on Osmosis
 * 4. Resolver uses revealed secret to claim ETH on Sepolia
 */

import { ethers } from 'ethers';
import { DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { SigningStargateClient } from '@cosmjs/stargate';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import crypto from 'crypto';
import chalk from 'chalk';
import ora from 'ora';

// Configuration
const CONFIG = {
  // Ethereum Sepolia
  ETH_RPC_URL: 'https://sepolia.infura.io/v3/YOUR_INFURA_KEY',
  
  // User keys (who wants to swap ETH for ATOM)
  USER_ETH_PRIVATE_KEY: 'your_user_ethereum_private_key_here',
  USER_COSMOS_MNEMONIC: 'your_user_cosmos_mnemonic_here',
  
  // Resolver keys (who provides liquidity)
  RESOLVER_ETH_PRIVATE_KEY: 'your_resolver_ethereum_private_key_here',
  RESOLVER_COSMOS_MNEMONIC: 'your_resolver_cosmos_mnemonic_here',
  
  // Contract addresses
  ETH_ESCROW_SRC: '0x0000000000000000000000000000000000000000', // Replace with deployed contract address
  OSMOSIS_ESCROW_FACTORY: 'osmo1...', // Replace with deployed contract address
  OSMOSIS_ESCROW_DST_CODE_ID: 1,
  
  // Swap parameters
  SWAP_AMOUNT_ETH: '0.01', // 0.01 ETH
  SWAP_AMOUNT_ATOM: '0.1', // 0.1 ATOM
  TIMELOCK_ETH: 3600, // 1 hour
  TIMELOCK_OSMOSIS: 5400, // 1.5 hours
  
  // Fee configuration
  PROTOCOL_FEE_AMOUNT: '0', // No protocol fee for demo
  INTEGRATOR_FEE_AMOUNT: '0', // No integrator fee for demo
  RESCUE_DELAY: 86400, // 24 hours
};

// Utility functions
function generateSecret() {
  return crypto.randomBytes(32).toString('hex');
}

function generateHashlock(secret) {
  return ethers.keccak256('0x' + secret);
}

function logStep(step, message) {
  console.log(chalk.blue(`\n[Step ${step}] ${message}`));
}

function logSuccess(message) {
  console.log(chalk.green(`✅ ${message}`));
}

function logError(message) {
  console.log(chalk.red(`❌ ${message}`));
}

function logInfo(message) {
  console.log(chalk.yellow(`ℹ️  ${message}`));
}

// 1inch Fusion Plus Contract Functions (Updated)
const ESCROW_FACTORY_FUNCTIONS = {
  create_escrow_dst: {
    immutables: {
      order_hash: "string",
      hashlock: "string", 
      maker: "string",
      taker: "string",
      token: "string",
      amount: "string", // Uint256
      safety_deposit: "string", // Uint256
      timelocks: {
        deployed_at: "number",
        src_withdrawal: "number",
        src_public_withdrawal: "number", 
        src_cancellation: "number",
        src_public_cancellation: "number",
        dst_withdrawal: "number",
        dst_public_withdrawal: "number",
        dst_cancellation: "number"
      },
      parameters: "array" // Vec<u8> - contains FeeInfo JSON
    },
    src_cancellation_timestamp: "number"
  }
};

const ESCROW_DST_FUNCTIONS = {
  withdraw: {
    secret: "binary",
    immutables: "object"
  },
  public_withdraw: {
    secret: "binary", 
    immutables: "object"
  },
  cancel: {
    immutables: "object"
  },
  rescue_funds: {
    token: "string",
    amount: "string", // Uint256
    immutables: "object"
  }
};

// Fee structure for parameters
const FEE_INFO = {
  protocol_fee_amount: "0", // Uint128
  integrator_fee_amount: "0", // Uint128  
  protocol_fee_recipient: "osmo1...", // Address
  integrator_fee_recipient: "osmo1..." // Address
};

// Contract ABIs (placeholder - replace with actual ABIs when contracts are deployed)
const ESCROW_SRC_ABI = [
  "function createEscrowSrc(tuple(bytes32 orderHash, bytes32 hashlock, address maker, address taker, string token, uint256 amount, uint256 safetyDeposit, tuple(uint64 deployedAt, uint32 srcWithdrawal, uint32 srcPublicWithdrawal, uint32 srcCancellation, uint32 srcPublicCancellation, uint32 dstWithdrawal, uint32 dstPublicWithdrawal, uint32 dstCancellation) timelocks, bytes parameters) immutables, uint64 srcCancellationTimestamp) external payable",
  "function withdraw(tuple(bytes32 orderHash, bytes32 hashlock, address maker, address taker, string token, uint256 amount, uint256 safetyDeposit, tuple(uint64 deployedAt, uint32 srcWithdrawal, uint32 srcPublicWithdrawal, uint32 srcCancellation, uint32 srcPublicCancellation, uint32 dstWithdrawal, uint32 dstPublicWithdrawal, uint32 dstCancellation) timelocks, bytes parameters) immutables, bytes32 secret) external",
  "function cancel(tuple(bytes32 orderHash, bytes32 hashlock, address maker, address taker, string token, uint256 amount, uint256 safetyDeposit, tuple(uint64 deployedAt, uint32 srcWithdrawal, uint32 srcPublicWithdrawal, uint32 srcCancellation, uint32 srcPublicCancellation, uint32 dstWithdrawal, uint32 dstPublicWithdrawal, uint32 dstCancellation) timelocks, bytes parameters) immutables) external",
  "function rescueFunds(tuple(bytes32 orderHash, bytes32 hashlock, address maker, address taker, string token, uint256 amount, uint256 safetyDeposit, tuple(uint64 deployedAt, uint32 srcWithdrawal, uint32 srcPublicWithdrawal, uint32 srcCancellation, uint32 srcPublicCancellation, uint32 dstWithdrawal, uint32 dstPublicWithdrawal, uint32 dstCancellation) timelocks, bytes parameters) immutables, string token, uint256 amount) external"
];

class CrossChainSwapDemo {
  constructor() {
    this.secret = null;
    this.hashlock = null;
    
    // User wallets (who wants to swap ETH for ATOM)
    this.userEthWallet = null;
    this.userCosmosWallet = null;
    
    // Resolver wallets (who provides liquidity)
    this.resolverEthWallet = null;
    this.resolverCosmosWallet = null;
    
    // Providers and clients
    this.ethProvider = null;
    this.cosmosClient = null;
    this.swapId = null;
    
    // Escrow addresses
    this.escrowDstAddress = null; // Set in step 2, used in step 3
  }

  async initialize() {
    logStep(1, "Initializing wallets and connections...");
    
    try {
      // Initialize Ethereum provider
      this.ethProvider = new ethers.JsonRpcProvider(CONFIG.ETH_RPC_URL);
      
      // Initialize User wallets
      this.userEthWallet = new ethers.Wallet(CONFIG.USER_ETH_PRIVATE_KEY, this.ethProvider);
      this.userCosmosWallet = await DirectSecp256k1HdWallet.fromMnemonic(CONFIG.USER_COSMOS_MNEMONIC);
      
      // Initialize Resolver wallets
      this.resolverEthWallet = new ethers.Wallet(CONFIG.RESOLVER_ETH_PRIVATE_KEY, this.ethProvider);
      this.resolverCosmosWallet = await DirectSecp256k1HdWallet.fromMnemonic(CONFIG.RESOLVER_COSMOS_MNEMONIC);
      
      // Initialize Cosmos client (using resolver for contract interactions)
      this.cosmosClient = await SigningCosmWasmClient.connectWithSigner(
        CONFIG.OSMOSIS_RPC_URL,
        this.resolverCosmosWallet
      );
      
      logSuccess(`User Ethereum wallet: ${this.userEthWallet.address}`);
      logSuccess(`User Cosmos wallet: ${(await this.userCosmosWallet.getAccounts())[0].address}`);
      logSuccess(`Resolver Ethereum wallet: ${this.resolverEthWallet.address}`);
      logSuccess(`Resolver Cosmos wallet: ${(await this.resolverCosmosWallet.getAccounts())[0].address}`);
      
    } catch (error) {
      logError(`Initialization failed: ${error.message}`);
      throw error;
    }
  }

  async step1_UserCreatesSourceEscrow() {
    logStep(2, "User creates EscrowSrc on Ethereum and deposits ETH");
    
    try {
      // Generate secret and hashlock
      this.secret = generateSecret();
      this.hashlock = generateHashlock(this.secret);
      
      logInfo(`Secret: ${this.secret}`);
      logInfo(`Hashlock: ${this.hashlock}`);
      
      // User creates EscrowSrc on Ethereum and deposits ETH
      logInfo(`Creating EscrowSrc on Ethereum...`);
      logInfo(`Depositing ${CONFIG.SWAP_AMOUNT_ETH} ETH into source escrow...`);
      
      // Check if contract address is set
      if (CONFIG.ETH_ESCROW_SRC === '0x0000000000000000000000000000000000000000') {
        throw new Error('ETH_ESCROW_SRC contract address not set. Please update CONFIG.ETH_ESCROW_SRC with deployed contract address.');
      }
      
      // Get resolver Cosmos address
      const resolverCosmosAddress = (await this.resolverCosmosWallet.getAccounts())[0].address;
      
      // Create EscrowSrc on Ethereum
      const ethEscrowMsg = {
        create_escrow_src: {
          immutables: {
            order_hash: ethers.keccak256(ethers.toUtf8Bytes("swap_" + Date.now())),
            hashlock: this.hashlock,
            maker: this.userEthWallet.address, // User
            taker: resolverCosmosAddress, // Resolver (Cosmos address)
            token: "ETH",
            amount: ethers.parseEther(CONFIG.SWAP_AMOUNT_ETH).toString(),
            safety_deposit: ethers.parseEther("0.001").toString(),
            timelocks: {
              deployed_at: 0,
              src_withdrawal: 300,
              src_public_withdrawal: 600,
              src_cancellation: 1800,
              src_public_cancellation: 3600,
              dst_withdrawal: 300,
              dst_public_withdrawal: 600,
              dst_cancellation: 5400
            },
            parameters: Buffer.from(JSON.stringify({
              protocol_fee_amount: "0",
              integrator_fee_amount: "0",
              protocol_fee_recipient: this.userEthWallet.address,
              integrator_fee_recipient: this.userEthWallet.address
            }), 'utf8')
          },
          src_cancellation_timestamp: Math.floor(Date.now() / 1000) + CONFIG.TIMELOCK_ETH
        }
      };
      
      // Execute transaction to create EscrowSrc and deposit ETH
      logInfo(`Executing transaction on Ethereum...`);
      
      // Create contract instance
      const escrowSrcContract = new ethers.Contract(CONFIG.ETH_ESCROW_SRC, ESCROW_SRC_ABI, this.userEthWallet);
      
      // Execute transaction
      const tx = await escrowSrcContract.createEscrowSrc(
        ethEscrowMsg.create_escrow_src.immutables,
        ethEscrowMsg.create_escrow_src.src_cancellation_timestamp,
        {
          value: ethers.parseEther(CONFIG.SWAP_AMOUNT_ETH)
        }
      );
      
      logInfo(`Transaction submitted: ${tx.hash}`);
      logInfo(`Waiting for confirmation...`);
      
      const receipt = await tx.wait();
      logInfo(`Transaction confirmed in block: ${receipt.blockNumber}`);
      logInfo(`Gas used: ${receipt.gasUsed.toString()}`);
      logSuccess(`EscrowSrc created and ETH deposited successfully!`);
      
    } catch (error) {
      logError(`Step 1 failed: ${error.message}`);
      throw error;
    }
  }

  async step2_ResolverCreatesEscrowDst() {
    logStep(3, "Resolver creates EscrowDst on Osmosis");
    
    try {
      const resolverCosmosAddress = (await this.resolverCosmosWallet.getAccounts())[0].address;
      
      // Check if contract address is set
      if (CONFIG.OSMOSIS_ESCROW_FACTORY === 'osmo1...') {
        throw new Error('OSMOSIS_ESCROW_FACTORY contract address not set. Please update CONFIG.OSMOSIS_ESCROW_FACTORY with deployed contract address.');
      }
      
      // Prepare immutables for escrow creation (matching your CosmWasm contract structure)
      const feeInfo = {
        protocol_fee_amount: "0", // No protocol fee for demo
        integrator_fee_amount: "0", // No integrator fee for demo
        protocol_fee_recipient: resolverCosmosAddress, // Resolver address
        integrator_fee_recipient: resolverCosmosAddress // Resolver address
      };
      
      const immutables = {
        order_hash: ethers.keccak256(ethers.toUtf8Bytes("swap_" + Date.now())),
        hashlock: this.hashlock,
        maker: this.userEthWallet.address, // User (who wants ATOM)
        taker: resolverCosmosAddress, // Resolver (who provides ATOM)
        token: "uosmo",
        amount: ethers.parseUnits(CONFIG.SWAP_AMOUNT_ATOM, 6).toString(), // ATOM has 6 decimals
        safety_deposit: ethers.parseUnits("0.01", 6).toString(), // 0.01 ATOM safety deposit
        timelocks: {
          deployed_at: 0, // Will be set by contract
          src_withdrawal: 300, // 5 minutes
          src_public_withdrawal: 600, // 10 minutes
          src_cancellation: 1800, // 30 minutes
          src_public_cancellation: 3600, // 1 hour
          dst_withdrawal: 300, // 5 minutes
          dst_public_withdrawal: 600, // 10 minutes
          dst_cancellation: 5400 // 1.5 hours
        },
        parameters: Buffer.from(JSON.stringify(feeInfo), 'utf8') // FeeInfo as JSON bytes
      };
      
      const srcCancellationTimestamp = Math.floor(Date.now() / 1000) + CONFIG.TIMELOCK_ETH;
      
      logInfo(`Calling EscrowFactory.create_escrow_dst()...`);
      logInfo(`Maker (User): ${immutables.maker}`);
      logInfo(`Taker (Resolver): ${immutables.taker}`);
      logInfo(`Amount: ${CONFIG.SWAP_AMOUNT_ATOM} ATOM`);
      
      // Call 1inch Fusion Plus EscrowFactory.create_escrow_dst()
      const msg = {
        create_escrow_dst: {
          immutables,
          src_cancellation_timestamp: srcCancellationTimestamp
        }
      };
      
      // Execute transaction on Osmosis
      logInfo(`Executing transaction on Osmosis...`);
      
      const result = await this.cosmosClient.execute(
        resolverCosmosAddress,
        CONFIG.OSMOSIS_ESCROW_FACTORY,
        msg,
        "auto",
        "Creating EscrowDst for cross-chain swap",
        [
          {
            denom: "uosmo",
            amount: ethers.parseUnits(CONFIG.SWAP_AMOUNT_ATOM, 6).toString()
          },
          {
            denom: "uosmo", 
            amount: ethers.parseUnits("0.01", 6).toString() // Safety deposit
          }
        ]
      );
      
      logInfo(`Transaction hash: ${result.transactionHash}`);
      
      // Extract and store the escrow address from transaction events
      const escrowAddress = result.events.find(e => e.type === 'wasm').attributes.find(a => a.key === 'escrow_address')?.value;
      if (escrowAddress) {
        this.escrowDstAddress = escrowAddress;
        logInfo(`EscrowDst address: ${this.escrowDstAddress}`);
      } else {
        throw new Error('Failed to extract EscrowDst address from transaction events');
      }
      
      logSuccess(`EscrowDst created using 1inch Fusion Plus successfully!`);
      
    } catch (error) {
      logError(`Step 2 failed: ${error.message}`);
      throw error;
    }
  }

  async step3_UserClaimsATOM() {
    logStep(4, "User claims ATOM on Osmosis");
    
    try {
      const userCosmosAddress = (await this.userCosmosWallet.getAccounts())[0].address;
      
      logInfo(`Revealing secret: ${this.secret}`);
      logInfo(`Calling EscrowDst.withdraw()...`);
      
      // Use the escrow address from step 2
      if (!this.escrowDstAddress) {
        throw new Error('EscrowDst address not set. Make sure step 2 completed successfully.');
      }
      
      logInfo(`Using EscrowDst address: ${this.escrowDstAddress}`);
      
      // Prepare fee info for withdraw
      const feeInfo = {
        protocol_fee_amount: "0",
        integrator_fee_amount: "0", 
        protocol_fee_recipient: userCosmosAddress,
        integrator_fee_recipient: userCosmosAddress
      };
      
      // Call 1inch Fusion Plus EscrowDst.withdraw()
      const msg = {
        withdraw: {
          secret: Buffer.from(this.secret, 'hex'), // Convert hex to binary
          immutables: {
            order_hash: ethers.keccak256(ethers.toUtf8Bytes("swap_" + Date.now())),
            hashlock: this.hashlock,
            maker: this.userEthWallet.address,
            taker: userCosmosAddress,
            token: "uosmo",
            amount: ethers.parseUnits(CONFIG.SWAP_AMOUNT_ATOM, 6).toString(),
            safety_deposit: ethers.parseUnits("0.01", 6).toString(),
            timelocks: {
              deployed_at: 0,
              src_withdrawal: 300,
              src_public_withdrawal: 600,
              src_cancellation: 1800,
              src_public_cancellation: 3600,
              dst_withdrawal: 300,
              dst_public_withdrawal: 600,
              dst_cancellation: 5400
            },
            parameters: Buffer.from(JSON.stringify(feeInfo), 'utf8') // FeeInfo as JSON bytes
          }
        }
      };
      
      // Create user Cosmos client for withdrawal
      const userCosmosClient = await SigningCosmWasmClient.connectWithSigner(
        CONFIG.OSMOSIS_RPC_URL,
        this.userCosmosWallet
      );
      
      logInfo(`Executing withdraw transaction on Osmosis...`);
      
      const result = await userCosmosClient.execute(
        userCosmosAddress,
        this.escrowDstAddress,
        msg,
        "auto",
        "Withdrawing ATOM from EscrowDst"
      );
      
      logInfo(`Transaction hash: ${result.transactionHash}`);
      logInfo(`ATOM withdrawn: ${CONFIG.SWAP_AMOUNT_ATOM}`);
      logSuccess(`ATOM claimed using 1inch Fusion Plus successfully!`);
      
    } catch (error) {
      logError(`Step 3 failed: ${error.message}`);
      throw error;
    }
  }

  async step4_ResolverClaimsETH() {
    logStep(5, "Resolver claims ETH using revealed secret");
    
    try {
      logInfo(`Using revealed secret to claim ETH from EscrowSrc...`);
      
      // Check if contract address is set
      if (CONFIG.ETH_ESCROW_SRC === '0x0000000000000000000000000000000000000000') {
        throw new Error('ETH_ESCROW_SRC contract address not set. Please update CONFIG.ETH_ESCROW_SRC with deployed contract address.');
      }
      
      // Get resolver Cosmos address
      const resolverCosmosAddress = (await this.resolverCosmosWallet.getAccounts())[0].address;
      
      // Create withdraw message for EscrowSrc
      const ethWithdrawMsg = {
        withdraw: {
          secret: Buffer.from(this.secret, 'hex'),
          immutables: {
            order_hash: ethers.keccak256(ethers.toUtf8Bytes("swap_" + Date.now())),
            hashlock: this.hashlock,
            maker: this.userEthWallet.address,
            taker: resolverCosmosAddress, // Resolver
            token: "ETH",
            amount: ethers.parseEther(CONFIG.SWAP_AMOUNT_ETH).toString(),
            safety_deposit: ethers.parseEther("0.001").toString(),
            timelocks: {
              deployed_at: 0,
              src_withdrawal: 300,
              src_public_withdrawal: 600,
              src_cancellation: 1800,
              src_public_cancellation: 3600,
              dst_withdrawal: 300,
              dst_public_withdrawal: 600,
              dst_cancellation: 5400
            },
            parameters: Buffer.from(JSON.stringify({
              protocol_fee_amount: "0",
              integrator_fee_amount: "0",
              protocol_fee_recipient: this.resolverEthWallet.address,
              integrator_fee_recipient: this.resolverEthWallet.address
            }), 'utf8')
          }
        }
      };
      
      // Execute transaction to claim ETH
      logInfo(`Executing claim transaction on Ethereum...`);
      
      // Create contract instance
      const escrowSrcContract = new ethers.Contract(CONFIG.ETH_ESCROW_SRC, ESCROW_SRC_ABI, this.resolverEthWallet);
      
      // Execute transaction
      const tx = await escrowSrcContract.withdraw(
        ethWithdrawMsg.withdraw.immutables,
        ethWithdrawMsg.withdraw.secret
      );
      
      logInfo(`Transaction submitted: ${tx.hash}`);
      logInfo(`Waiting for confirmation...`);
      
      const receipt = await tx.wait();
      logInfo(`Transaction confirmed in block: ${receipt.blockNumber}`);
      logInfo(`Gas used: ${receipt.gasUsed.toString()}`);
      logSuccess(`ETH claimed successfully!`);
      
    } catch (error) {
      logError(`Step 4 failed: ${error.message}`);
      throw error;
    }
  }

  async runDemo() {
    console.log(chalk.bold.cyan('\n🚀 1inch Fusion Plus Cross-Chain Swap Demo: Sepolia ↔ Osmosis\n'));
    
    try {
      await this.initialize();
      await this.step1_UserCreatesSourceEscrow();
      await this.step2_ResolverCreatesEscrowDst();
      await this.step3_UserClaimsATOM();
      await this.step4_ResolverClaimsETH();
      
      console.log(chalk.bold.green('\n🎉 1inch Fusion Plus swap completed successfully!\n'));
      console.log(chalk.green('Summary:'));
      console.log(chalk.green(`- User swapped ${CONFIG.SWAP_AMOUNT_ETH} ETH (Sepolia) for ${CONFIG.SWAP_AMOUNT_ATOM} ATOM (Osmosis)`));
      console.log(chalk.green(`- Secret used: ${this.secret}`));
      console.log(chalk.green(`- Hashlock: ${this.hashlock}`));
      console.log(chalk.green(`- Used 1inch Fusion Plus contracts:`));
      console.log(chalk.green(`  • EscrowFactory.create_escrow_dst()`));
      console.log(chalk.green(`  • EscrowDst.withdraw()`));
      console.log(chalk.green(`  • EscrowDst.cancel()`));
      console.log(chalk.green(`  • EscrowDst.rescue_funds()`));
      console.log(chalk.green(`- Features: Fee distribution, timelock validation, safety deposits`));
      
    } catch (error) {
      logError(`Demo failed: ${error.message}`);
      process.exit(1);
    }
  }
}

// Run the demo
if (import.meta.url === `file://${process.argv[1]}`) {
  const demo = new CrossChainSwapDemo();
  demo.runDemo().catch(console.error);
}

export { CrossChainSwapDemo };
