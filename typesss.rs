import Candid;
import HashMap "mo:base/HashMap";
import HashSet "mo:base/HashSet";
import Principal "mo:base/Principal";

type Result<T = Nat, E = Error> = Result.T<Error, T>;

type LogoResult = { logo_type: Text; data: Text };

type MetadataVal = TextContent(Text) | BlobContent(Blob) | NatContent(Nat) | Nat8Content(Nat8) | Nat16Content(Nat16) | Nat32Content(Nat32) | Nat64Content(Nat64);

type MetadataPart = { purpose: MetadataPurpose; key_val_data: HashMap.HashMap<Text, MetadataVal>; data: Blob };

type MetadataDesc = Array<MetadataPart>;

type MetadataDescRef = [MetadataPart];

type Nft = { owner: Principal; approved: ?Principal; id: Nat64; metadata: MetadataDesc; content: Blob };

type State = {
  nfts: Array<Nft>;
  custodians: HashSet.HashSet<Principal>;
  operators: HashMap.HashMap<Principal, HashSet.HashSet<Principal>>;
  logo: ?LogoResult;
  name: Text;
  symbol: Text;
  txid: Nat;
};

type MintResult = { token_id: Nat64; id: Nat128 };

type ExtendedMetadataResult = { metadata_desc: MetadataDescRef; token_id: Nat64 };

type InterfaceId = Approval | TransactionHistory | Mint | Burn | TransferNotification;

type ConstrainedError = Unauthorized;

type InitArgs = { custodians: ?HashSet.HashSet<Principal>; logo: ?LogoResult; name: Text; symbol: Text };

actor NftContract {

  var state: State;

  public shared({}) func init(init_args: InitArgs) : async () {
    let initialState = {
      nfts = [];
      custodians = switch (init_args.custodians) {
        null => HashSet.empty();
        some(c) => c;
      };
      operators = HashMap.empty();
      logo = init_args.logo;
      name = init_args.name;
      symbol = init_args.symbol;
      txid = 0;
    };
    state := initialState;
  };

  public func balance_of(owner: Principal) : async Nat {
    let ownerNfts = [nft.id : nft in state.nfts where (nft.owner == owner)];
    return Array.length(ownerNfts);
  };

  public func owner_of(token_id: Nat64) : async Principal {
    let nft = switch (Array.find((nft) => nft.id == token_id, state.nfts)) {
      null => {
        // Token not found
        reject(Error.InvalidTokenId);
      };
      some(nft) => nft;
    };
    return nft.owner;
  };

  public shared func transfer_from(sender: Principal, to: Principal, token_id: Nat64) : async () {
    assert_can_transfer(sender, token_id);
    let nft = switch (Array.find((nft) => nft.id == token_id, state.nfts)) {
      null => {
        // Token not found
        reject(Error.InvalidTokenId);
      };
      some(nft) => nft;
    };
    nft.owner := to;
    if (Array.member(token_id, sender_nfts)) {
      sender_nfts := Array.filter((id) => id != token_id, sender_nfts);
    };
    if (!Array.member(token_id, to_nfts)) {
      to_nfts := Array.concat(to_nfts, [token_id]);
    };
  };

  public func total_supply() : async Nat {
    return Array.length(state.nfts);
  };

  public func get_metadata(token_id: Nat64) : async MetadataDescRef {
    let nft = switch (Array.find((nft) => nft.id == token_id, state.nfts)) {
      null => {
        // Token not found
        reject(Error.InvalidTokenId);
      };
      some(nft) => nft;
    };
    return nft.metadata;
  };

  public func get_metadata_for_user(owner: Principal) : async Array<ExtendedMetadataResult> {
    let ownerNfts = [nft : nft in state.nfts where (nft.owner == owner)];
    return [ { metadata_desc = nft.metadata; token_id = nft.id } : nft in ownerNfts ];
  };

  public shared({}) func approve(spender: Principal, token_id: Nat64) : async () {
    assert_can_approve(token_id);
    let nft = switch (Array.find((nft) => nft.id == token_id, state.nfts)) {
      null => {
        // Token not found
        reject(Error.InvalidTokenId);
      };
      some(nft) => nft;
    };
    nft.approved := spender;
  };

  public shared({}) func set_approval_for_all(operator: Principal, approved: bool) : async () {
    state.operators[caller] := HashSet.insert(operator, state.operators[caller]);
  };

  private func assert_can_transfer(sender: Principal, token_id: Nat64) : async () {
    let nft = switch (Array.find((nft) => nft.id == token_id, state.nfts)) {
      null => {
        // Token not found
        reject(Error.InvalidTokenId);
      };
      some(nft) => nft;
    };
    if (!(sender == nft.owner || nft.approved == sender || state.operators[nft.owner]?.contains(sender) : false)) {
      reject(Error.Unauthorized);
    };
  };

  private func assert_can_approve(token_id: Nat64) : async {
    let nft = switch (Array.find((nft) => nft.id == token_id, state.nfts)) {
      null => {
        // Token not found
        reject(Error.InvalidTokenId);
      };
      some(nft) => nft;
    };
    if (!(caller == nft.owner || state.operators[nft.owner]?.contains(caller) : false)) {
      reject(Error.Unauthorized);
    };
  };
};
