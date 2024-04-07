"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.DecentralizedStablecoinContract = exports.DecentralizedStablecoinQueryContract = void 0;
const wasmkit_1 = require("@kubiklabs/wasmkit");
class DecentralizedStablecoinQueryContract extends wasmkit_1.Contract {
    constructor(contractName, instantiateTag) {
        super(contractName, instantiateTag);
        this.balance = async ({ address }) => {
            return this.queryMsg({
                balance: {
                    address
                }
            });
        };
        this.tokenInfo = async () => {
            return this.queryMsg({
                token_info: {}
            });
        };
        this.minter = async () => {
            return this.queryMsg({
                minter: {}
            });
        };
        this.allowance = async ({ owner, spender }) => {
            return this.queryMsg({
                allowance: {
                    owner,
                    spender
                }
            });
        };
        this.allAllowances = async ({ limit, owner, startAfter }) => {
            return this.queryMsg({
                all_allowances: {
                    limit,
                    owner,
                    start_after: startAfter
                }
            });
        };
        this.allSpenderAllowances = async ({ limit, spender, startAfter }) => {
            return this.queryMsg({
                all_spender_allowances: {
                    limit,
                    spender,
                    start_after: startAfter
                }
            });
        };
        this.allAccounts = async ({ limit, startAfter }) => {
            return this.queryMsg({
                all_accounts: {
                    limit,
                    start_after: startAfter
                }
            });
        };
        this.marketingInfo = async () => {
            return this.queryMsg({
                marketing_info: {}
            });
        };
        this.downloadLogo = async () => {
            return this.queryMsg({
                download_logo: {}
            });
        };
        this.balance = this.balance.bind(this);
        this.tokenInfo = this.tokenInfo.bind(this);
        this.minter = this.minter.bind(this);
        this.allowance = this.allowance.bind(this);
        this.allAllowances = this.allAllowances.bind(this);
        this.allSpenderAllowances = this.allSpenderAllowances.bind(this);
        this.allAccounts = this.allAccounts.bind(this);
        this.marketingInfo = this.marketingInfo.bind(this);
        this.downloadLogo = this.downloadLogo.bind(this);
    }
}
exports.DecentralizedStablecoinQueryContract = DecentralizedStablecoinQueryContract;
class DecentralizedStablecoinContract extends DecentralizedStablecoinQueryContract {
    constructor(instantiateTag) {
        super("decentralized_stablecoin", instantiateTag);
        this.transfer = async ({ account, customFees, memo, transferAmount }, { amount, recipient }) => {
            return await this.executeMsg({
                transfer: {
                    amount,
                    recipient
                }
            }, account, customFees, memo, transferAmount);
        };
        this.burn = async ({ account, customFees, memo, transferAmount }, { amount }) => {
            return await this.executeMsg({
                burn: {
                    amount
                }
            }, account, customFees, memo, transferAmount);
        };
        this.send = async ({ account, customFees, memo, transferAmount }, { amount, contract, msg }) => {
            return await this.executeMsg({
                send: {
                    amount,
                    contract,
                    msg
                }
            }, account, customFees, memo, transferAmount);
        };
        this.increaseAllowance = async ({ account, customFees, memo, transferAmount }, { amount, expires, spender }) => {
            return await this.executeMsg({
                increase_allowance: {
                    amount,
                    expires,
                    spender
                }
            }, account, customFees, memo, transferAmount);
        };
        this.decreaseAllowance = async ({ account, customFees, memo, transferAmount }, { amount, expires, spender }) => {
            return await this.executeMsg({
                decrease_allowance: {
                    amount,
                    expires,
                    spender
                }
            }, account, customFees, memo, transferAmount);
        };
        this.transferFrom = async ({ account, customFees, memo, transferAmount }, { amount, owner, recipient }) => {
            return await this.executeMsg({
                transfer_from: {
                    amount,
                    owner,
                    recipient
                }
            }, account, customFees, memo, transferAmount);
        };
        this.sendFrom = async ({ account, customFees, memo, transferAmount }, { amount, contract, msg, owner }) => {
            return await this.executeMsg({
                send_from: {
                    amount,
                    contract,
                    msg,
                    owner
                }
            }, account, customFees, memo, transferAmount);
        };
        this.burnFrom = async ({ account, customFees, memo, transferAmount }, { amount, owner }) => {
            return await this.executeMsg({
                burn_from: {
                    amount,
                    owner
                }
            }, account, customFees, memo, transferAmount);
        };
        this.mint = async ({ account, customFees, memo, transferAmount }, { amount, recipient }) => {
            return await this.executeMsg({
                mint: {
                    amount,
                    recipient
                }
            }, account, customFees, memo, transferAmount);
        };
        this.updateMinter = async ({ account, customFees, memo, transferAmount }, { newMinter }) => {
            return await this.executeMsg({
                update_minter: {
                    new_minter: newMinter
                }
            }, account, customFees, memo, transferAmount);
        };
        this.updateMarketing = async ({ account, customFees, memo, transferAmount }, { description, marketing, project }) => {
            return await this.executeMsg({
                update_marketing: {
                    description,
                    marketing,
                    project
                }
            }, account, customFees, memo, transferAmount);
        };
        this.uploadLogo = async ({ account, customFees, memo, transferAmount }) => {
            return await this.executeMsg({
                upload_logo: {}
            }, account, customFees, memo, transferAmount);
        };
        this.transfer = this.transfer.bind(this);
        this.burn = this.burn.bind(this);
        this.send = this.send.bind(this);
        this.increaseAllowance = this.increaseAllowance.bind(this);
        this.decreaseAllowance = this.decreaseAllowance.bind(this);
        this.transferFrom = this.transferFrom.bind(this);
        this.sendFrom = this.sendFrom.bind(this);
        this.burnFrom = this.burnFrom.bind(this);
        this.mint = this.mint.bind(this);
        this.updateMinter = this.updateMinter.bind(this);
        this.updateMarketing = this.updateMarketing.bind(this);
        this.uploadLogo = this.uploadLogo.bind(this);
    }
}
exports.DecentralizedStablecoinContract = DecentralizedStablecoinContract;
//# sourceMappingURL=DecentralizedStablecoinContract.js.map