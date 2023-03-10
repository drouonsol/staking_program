import * as anchor from "@project-serum/anchor"
import { Program } from "@project-serum/anchor"
import { AnchorNftStaking } from "../target/types/anchor_nft_staking"
import { setupNft } from "./utils/setupNft"
import { PROGRAM_ID as METADATA_PROGRAM_ID } from "@metaplex-foundation/mpl-token-metadata"
import { expect } from "chai"
import { getAccount } from "@solana/spl-token"
import { findCandyMachinesByPublicKeyFieldOperation } from "@metaplex-foundation/js"

describe("anchor-nft-staking", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider)

  const program = anchor.workspace.AnchorNftStaking as Program<AnchorNftStaking>

  const wallet = anchor.workspace.AnchorNftStaking.provider.wallet

  let delegatedAuthPda: anchor.web3.PublicKey
  let stakeStatePda: anchor.web3.PublicKey
  let nft: any
  let mintAuth: anchor.web3.PublicKey
  let mint: anchor.web3.PublicKey
  let tokenAddress: anchor.web3.PublicKey

  before(async () => {
    ;({ nft, delegatedAuthPda, stakeStatePda, mint, mintAuth, tokenAddress } =
      await setupNft(program, wallet.payer))
  })

  it("Stakes", async () => {
    // Add your test here.
    const stake = await program.methods
      .stake()
      .accounts({
        nftTokenAccount: nft.tokenAddress,
        nftMint: nft.mintAddress,
        nftEdition: nft.masterEditionAddress,
        metadataProgram: METADATA_PROGRAM_ID,
      })
      .rpc()

      const [stakeStatePda] = await anchor.web3.PublicKey.findProgramAddress(
        [wallet.publicKey.toBuffer(), Buffer.from("elysianstake")],
        program.programId
      )

    const account = await program.account.userStakeInfo.fetch(stakeStatePda)

    const [NftStatePda] = await anchor.web3.PublicKey.findProgramAddress(
      [wallet.publicKey.toBuffer(), nft.tokenAddress.toBuffer()],
      program.programId
    )

  const nftstate = await program.account.nftStakeInfo.fetch(NftStatePda)
   console.log("Stake State :", nftstate.skateState)
    console.log("Tokens Owed :", account.tokensOwed.toNumber())
    console.log("???? Staking Transaction Signature : ",stake)
  });

  it("Redeems", async () => {
  const rewards = await program.methods
      .redeem()
      .accounts({
        nftTokenAccount: nft.tokenAddress,
        stakeMint: mint,
        userStakeAta: tokenAddress,
      })
      .rpc()

    const account = await program.account.userStakeInfo.fetch(stakeStatePda)

    const tokenAccount = await getAccount(provider.connection, tokenAddress)
    const [NftStatePda] = await anchor.web3.PublicKey.findProgramAddress(
      [wallet.publicKey.toBuffer(), nft.tokenAddress.toBuffer()],
      program.programId
    )

      

  const nftstate = await program.account.nftStakeInfo.fetch(NftStatePda)
  console.log("Stake State :", nftstate.skateState)
    console.log("???? Claiming Rewards Signature : ",rewards)
  })

  it("Unstakes", async () => {
    const unstake = await program.methods
      .unstake()
      .accounts({
        nftTokenAccount: nft.tokenAddress,
        nftMint: nft.mintAddress,
        nftEdition: nft.masterEditionAddress,
        metadataProgram: METADATA_PROGRAM_ID,
        stakeMint: mint,
        userStakeAta: tokenAddress,
      })
      .rpc()

    const account = await program.account.userStakeInfo.fetch(stakeStatePda)
    const [NftStatePda] = await anchor.web3.PublicKey.findProgramAddress(
      [wallet.publicKey.toBuffer(), nft.tokenAddress.toBuffer()],
      program.programId
    )
    const nftstate = await program.account.nftStakeInfo.fetch(NftStatePda)
  var stateoftoken = "Null"
 
      

    console.log("Stake State :", nftstate.skateState)
      console.log(" Unstake Signature : ", unstake)
  })
})
