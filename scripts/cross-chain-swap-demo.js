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
  ETH_PRIVATE_KEY: 'your_ethereum_private_key_here',
  ETH_HTLC_ADDRESS: '0x0000000000000000000000000000000000000000', // Will be deployed
  
  // Osmosis Testnet
  OSMOSIS_RPC_URL: 'https://rpc-test.osmosis.zone',
  OSMOSIS_MNEMONIC: 'your_cosmos_mnemonic_here',
  OSMOSIS_ESCROW_FACTORY: 'osmo1...', // Will be deployed
  OSMOSIS_ESCROW_CODE_ID: 1,
  
  // Swap parameters
  SWAP_AMOUNT_ETH: '0.01', // 0.01 ETH
  SWAP_AMOUNT_ATOM: '0.1', // 0.1 ATOM
  TIMELOCK_ETH: 3600, // 1 hour
  TIMELOCK_OSMOSIS: 5400, // 1.5 hours
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

// Contract ABIs (simplified placeholders)
const HTLC_ABI = [
  "function lockFunds(bytes32 hashlock, uint256 timelock) external payable",
  "function claimFunds(bytes32 secret) external",
  "function refund() external",
  "function getSwap(bytes32 hashlock) external view returns (address, uint256, uint256, bool, bool)",
  "event FundsLocked(bytes32 indexed hashlock, address indexed sender, uint256 amount, uint256 timelock)",
  "event FundsClaimed(bytes32 indexed hashlock, address indexed claimer, bytes32 secret)",
  "event FundsRefunded(bytes32 indexed hashlock, address indexed sender)"
];

const ESCROW_FACTORY_ABI = [
  "function createEscrowDst(tuple(bytes32 orderHash, bytes32 hashlock, address maker, address taker, string token, uint256 amount, uint256 safetyDeposit, tuple(uint64 deployedAt, uint32 srcWithdrawal, uint32 srcPublicWithdrawal, uint32 srcCancellation, uint32 srcPublicCancellation, uint32 dstWithdrawal, uint32 dstPublicWithdrawal, uint32 dstCancellation) timelocks, bytes parameters) immutables, uint64 srcCancellationTimestamp) external",
  "function addressOfEscrowDst(tuple(bytes32 orderHash, bytes32 hashlock, address maker, address taker, string token, uint256 amount, uint256 safetyDeposit, tuple(uint64 deployedAt, uint32 srcWithdrawal, uint32 srcPublicWithdrawal, uint32 srcCancellation, uint32 srcPublicCancellation, uint32 dstWithdrawal, uint32 dstPublicWithdrawal, uint32 dstCancellation) timelocks, bytes parameters) immutables) external view returns (address)"
];

const ESCROW_DST_ABI = [
  "function withdraw(bytes32 secret) external",
  "function publicWithdraw() external",
  "function cancel() external",
  "function getEscrowInfo() external view returns (address, uint256, uint256, bool, bool)"
];

class CrossChainSwapDemo {
  constructor() {
    this.secret = null;
    this.hashlock = null;
    this.ethWallet = null;
    this.cosmosWallet = null;
    this.ethProvider = null;
    this.cosmosClient = null;
    this.swapId = null;
  }

  async initialize() {
    logStep(1, "Initializing wallets and connections...");
    
    try {
      // Initialize Ethereum wallet
      this.ethWallet = new ethers.Wallet(CONFIG.ETH_PRIVATE_KEY);
      this.ethProvider = new ethers.JsonRpcProvider(CONFIG.ETH_RPC_URL);
      this.ethWallet = this.ethWallet.connect(this.ethProvider);
      
      // Initialize Cosmos wallet
      this.cosmosWallet = await DirectSecp256k1HdWallet.fromMnemonic(CONFIG.OSMOSIS_MNEMONIC);
      this.cosmosClient = await SigningCosmWasmClient.connectWithSigner(
        CONFIG.OSMOSIS_RPC_URL,
        this.cosmosWallet
      );
      
      logSuccess(`Ethereum wallet: ${this.ethWallet.address}`);
      logSuccess(`Cosmos wallet: ${(await this.cosmosWallet.getAccounts())[0].address}`);
      
    } catch (error) {
      logError(`Initialization failed: ${error.message}`);
      throw error;
    }
  }

  async step1_UserCreatesSwap() {
    logStep(2, "User creates swap and locks ETH on Sepolia");
    
    try {
      // Generate secret and hashlock
      this.secret = generateSecret();
      this.hashlock = generateHashlock(this.secret);
      
      logInfo(`Secret: ${this.secret}`);
      logInfo(`Hashlock: ${this.hashlock}`);
      
      // Calculate timelock
      const timelock = Math.floor(Date.now() / 1000) + CONFIG.TIMELOCK_ETH;
      
      // Create HTLC contract instance
      const htlcContract = new ethers.Contract(CONFIG.ETH_HTLC_ADDRESS, HTLC_ABI, this.ethWallet);
      
      // Lock funds (placeholder - will be implemented when contract is deployed)
      logInfo(`Locking ${CONFIG.SWAP_AMOUNT_ETH} ETH on Sepolia...`);
      logInfo(`Timelock expires at: ${new Date(timelock * 1000).toISOString()}`);
      
      // TODO: Implement actual contract call when HTLC is deployed
      // const tx = await htlcContract.lockFunds(this.hashlock, timelock, {
      //   value: ethers.parseEther(CONFIG.SWAP_AMOUNT_ETH)
      // });
      // await tx.wait();
      
      logSuccess(`ETH locked on Sepolia (simulated)`);
      
    } catch (error) {
      logError(`Step 1 failed: ${error.message}`);
      throw error;
    }
  }

  async step2_ResolverCreatesEscrow() {
    logStep(3, "Resolver creates escrow on Osmosis and deposits ATOM");
    
    try {
      const cosmosAddress = (await this.cosmosWallet.getAccounts())[0].address;
      
      // Prepare immutables for escrow creation
      const immutables = {
        orderHash: ethers.keccak256(ethers.toUtf8Bytes("swap_" + Date.now())),
        hashlock: this.hashlock,
        maker: this.ethWallet.address,
        taker: cosmosAddress,
        token: "uosmo",
        amount: ethers.parseUnits(CONFIG.SWAP_AMOUNT_ATOM, 6), // ATOM has 6 decimals
        safetyDeposit: ethers.parseUnits("0.01", 6), // 0.01 ATOM safety deposit
        timelocks: {
          deployedAt: 0, // Will be set by contract
          srcWithdrawal: 300, // 5 minutes
          srcPublicWithdrawal: 600, // 10 minutes
          srcCancellation: 1800, // 30 minutes
          srcPublicCancellation: 3600, // 1 hour
          dstWithdrawal: 300, // 5 minutes
          dstPublicWithdrawal: 600, // 10 minutes
          dstCancellation: 5400 // 1.5 hours
        },
        parameters: new Uint8Array(0)
      };
      
      const srcCancellationTimestamp = Math.floor(Date.now() / 1000) + CONFIG.TIMELOCK_ETH;
      
      logInfo(`Creating escrow on Osmosis...`);
      logInfo(`Maker: ${immutables.maker}`);
      logInfo(`Taker: ${immutables.taker}`);
      logInfo(`Amount: ${CONFIG.SWAP_AMOUNT_ATOM} ATOM`);
      
      // TODO: Implement actual contract call when escrow factory is deployed
      // const msg = {
      //   create_escrow_dst: {
      //     immutables,
      //     src_cancellation_timestamp: srcCancellationTimestamp
      //   }
      // };
      // const result = await this.cosmosClient.execute(
      //   cosmosAddress,
      //   CONFIG.OSMOSIS_ESCROW_FACTORY,
      //   msg,
      //   "auto"
      // );
      
      logSuccess(`Escrow created on Osmosis (simulated)`);
      
    } catch (error) {
      logError(`Step 2 failed: ${error.message}`);
      throw error;
    }
  }

  async step3_UserClaimsATOM() {
    logStep(4, "User reveals secret and claims ATOM on Osmosis");
    
    try {
      const cosmosAddress = (await this.cosmosWallet.getAccounts())[0].address;
      
      logInfo(`Revealing secret: ${this.secret}`);
      logInfo(`Claiming ATOM on Osmosis...`);
      
      // TODO: Implement actual contract call when escrow is deployed
      // const msg = {
      //   withdraw: {
      //     secret: this.secret
      //   }
      // };
      // const result = await this.cosmosClient.execute(
      //   cosmosAddress,
      //   escrowAddress, // Address from step 2
      //   msg,
      //   "auto"
      // );
      
      logSuccess(`ATOM claimed on Osmosis (simulated)`);
      
    } catch (error) {
      logError(`Step 3 failed: ${error.message}`);
      throw error;
    }
  }

  async step4_ResolverClaimsETH() {
    logStep(5, "Resolver uses revealed secret to claim ETH on Sepolia");
    
    try {
      logInfo(`Using revealed secret to claim ETH...`);
      
      // Create HTLC contract instance
      const htlcContract = new ethers.Contract(CONFIG.ETH_HTLC_ADDRESS, HTLC_ABI, this.ethWallet);
      
      // TODO: Implement actual contract call when HTLC is deployed
      // const tx = await htlcContract.claimFunds(this.secret);
      // await tx.wait();
      
      logSuccess(`ETH claimed on Sepolia (simulated)`);
      
    } catch (error) {
      logError(`Step 4 failed: ${error.message}`);
      throw error;
    }
  }

  async runDemo() {
    console.log(chalk.bold.cyan('\n🚀 Cross-Chain Swap Demo: Sepolia ↔ Osmosis\n'));
    
    try {
      await this.initialize();
      await this.step1_UserCreatesSwap();
      await this.step2_ResolverCreatesEscrow();
      await this.step3_UserClaimsATOM();
      await this.step4_ResolverClaimsETH();
      
      console.log(chalk.bold.green('\n🎉 Cross-chain swap completed successfully!\n'));
      console.log(chalk.green('Summary:'));
      console.log(chalk.green(`- User swapped ${CONFIG.SWAP_AMOUNT_ETH} ETH (Sepolia) for ${CONFIG.SWAP_AMOUNT_ATOM} ATOM (Osmosis)`));
      console.log(chalk.green(`- Secret used: ${this.secret}`));
      console.log(chalk.green(`- Hashlock: ${this.hashlock}`));
      
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
