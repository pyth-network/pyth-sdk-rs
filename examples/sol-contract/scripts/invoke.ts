const web3 = require("@solana/web3.js");
const {struct, b, u8, u32} = require("@solana/buffer-layout");

const admin = web3.Keypair.fromSecretKey(new Uint8Array([223,143,8,70,205,100,4,197,222,158,132,43,89,182,188,243,24,213,136,120,189,209,235,13,167,45,132,41,17,243,58,158,114,230,85,178,27,22,80,213,200,96,166,64,152,163,191,112,35,197,55,219,24,254,117,129,227,39,37,232,106,30,178,193]))

export const invoke = async (loan: string, collateral: string) => {
    let contract = admin.publicKey;
    let conn = new web3.Connection(web3.clusterApiUrl('devnet'));

    /* Prepare the payer account */
    console.log("Airdropping...");
    let payer = web3.Keypair.generate();
    let airdropSig = await conn.requestAirdrop(
        payer.publicKey, web3.LAMPORTS_PER_SOL
    );
    await conn.confirmTransaction(airdropSig);

    /* createInst is an instruction creating an account storing the
     * LoanInfo data, which will be passed to Init for initialization*/
    let sizeofLoanInfo = 1 + 32 + 8 + 32 + 8;
    let dataAccount = web3.Keypair.generate();
    let cost = await conn.getMinimumBalanceForRentExemption(sizeofLoanInfo);
    const createInst = web3.SystemProgram.createAccount({
        programId: contract,
        fromPubkey: payer.publicKey,
        space: sizeofLoanInfo,
        newAccountPubkey: dataAccount.publicKey,
        lamports: cost,
    });

    /* Specify accounts being touched by Init */
    const loanKey = new web3.PublicKey(loan);
    const collateralKey = new web3.PublicKey(collateral);
    let keys = [{pubkey: contract, isSigner: true, isWritable: false},
                {pubkey: dataAccount.publicKey, isSigner: false, isWritable: false},
                {pubkey: loanKey, isSigner: false, isWritable: false},
                {pubkey: collateralKey, isSigner: false, isWritable: false},
                ];

    /* Prepare parameters for Invoking the contract */
    let dataLayout = struct([ u8('instruction') ])
    let data = Buffer.alloc(dataLayout.span);
    dataLayout.encode(Object.assign({instruction: 1}), data);

    /* Invoke the Init instruction */
    console.log("Creating data account and invoking Init...");
    let txInit = new web3.Transaction({ feePayer: payer.publicKey });
    txInit.add(
        createInst, 
        new web3.TransactionInstruction({
            keys,
            programId: contract,
            data
        })
    );
    let txInitSig = await web3.sendAndConfirmTransaction(conn, txInit, [payer, dataAccount, admin]);
    console.log("TxHash: " + txInitSig);

    /* Invoke the Loan2Value instruction */
    dataLayout.encode(Object.assign({instruction: 0}), data);
    console.log("Checking loan to value ratio...");
    let txCheck = new web3.Transaction({ feePayer: payer.publicKey });
    txCheck.add(
        new web3.TransactionInstruction({
            keys,
            programId: contract,
            data
        })
    );
    let txCheckSig = await web3.sendAndConfirmTransaction(conn, txCheck, [payer, admin]);
    console.log("TxHash: " + txCheckSig);    
}

let ethToUSD = "EdVCmQ9FSPcVe5YySXDPCRmc8aDQLKJ9xvYBMZPie1Vw";
let usdtToUSD = "38xoQ4oeJCBrcVvca2cGk7iV1dAfrmTR1kmhSCJQ8Jto";
invoke(ethToUSD, usdtToUSD);
