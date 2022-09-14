const web3 = require("@solana/web3.js");
const {struct, u8} = require("@solana/buffer-layout");

export const invoke = async (loan: string, collateral: string) => {
    /* Airdrop */
    console.log("Airdroping...");
    let payer = web3.Keypair.generate();
    let keypair = web3.Keypair.generate();
    let conn = new web3.Connection(web3.clusterApiUrl('devnet'));
    let airdropSig = await conn.requestAirdrop(
        payer.publicKey,
        web3.LAMPORTS_PER_SOL,
    );
    await conn.confirmTransaction(airdropSig);

    /* Specify accounts being touched */
    const loanKey = new web3.PublicKey(loan);
    const collateralKey = new web3.PublicKey(collateral);
    let keys = [{pubkey: loanKey, isSigner: false, isWritable: false},
                {pubkey: collateralKey, isSigner: false, isWritable: false},
                {pubkey: keypair.publicKey, isSigner: true, isWritable: false}];

    /* Prepare parameters */
    let allocateStruct = {
        index: 0,     // Loan2Value is instruction #0 in the program
        layout: struct([
            u8('instruction'),
        ])
    };
    let data = Buffer.alloc(allocateStruct.layout.span);
    let layoutFields = Object.assign({instruction: allocateStruct.index});
    allocateStruct.layout.encode(layoutFields, data);
    
    /* Invoke transaction */
    let tx = new web3.Transaction({
        feePayer: payer.publicKey
    });
    let contract = process.argv[2];
    console.log("Invoking contract " + contract + "...");
    tx.add(new web3.TransactionInstruction({
        keys,
        programId: contract,
        data
    }));

    let txSig = await web3.sendAndConfirmTransaction(conn, tx, [payer, keypair]);
    console.log("Confirmed TxHash " + txSig);
}

let ethToUSD = "EdVCmQ9FSPcVe5YySXDPCRmc8aDQLKJ9xvYBMZPie1Vw";
let usdtToUSD = "38xoQ4oeJCBrcVvca2cGk7iV1dAfrmTR1kmhSCJQ8Jto";
invoke(ethToUSD, usdtToUSD);
