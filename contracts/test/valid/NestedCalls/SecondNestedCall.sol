pragma solidity ^0.4.17;

import "../../../BeakerContract.sol";

contract SecondNestedCall  is BeakerContract {


     // FirstNestedCall - store at 0x8001
     //   SecondNestedCall - store at 0x8002
     //     ThirdNestedCall - store at 0x8003
     //       FourthNestedCall - store at 0x8004
     //     FifthNestedCall - store at 0x8005
     //   SixthNestedCall - store at 0x8006
     // End
    function G() public {
        // First we do the store for FirstNestedCall
        write(1, 0x8002, 76);

        // Being our call to ThirdNestedCall
        bytes24 reqProc = bytes24("ThirdNestedCall");
        string memory fselector = "G()";
        assembly {
            function malloc(size) -> result {
                // align to 32-byte words
                let rsize := add(size,sub(32,mod(size,32)))
                // get the current free mem location
                result :=  mload(0x40)
                // Bump the value of 0x40 so that it holds the next
                // available memory location.
                mstore(0x40,add(result,rsize))
            }
            let ins := malloc(128)
            // First set up the input data (at memory location 0x0)
            // The call call is 0x-03
            mstore(add(ins,0x0),0x03)
            // The capability index is 0x-02
            mstore(add(ins,0x20),0x02)
            // The key of the procedure
            mstore(add(ins,0x40),reqProc)
            // The size of the return value we expect (0x20)
            let retSize := 0x20
            let retLoc := malloc(retSize)
            mstore(add(ins,0x60),retSize)
            mstore(add(ins,0x80),keccak256(add(fselector,0x20),mload(fselector)))
            // "in_offset" is at 31, because we only want the last byte of type
            // "in_size" is 65 because it is 1+32+32+32+4
            // we will store the result at 0x80 and it will be 32 bytes
            if iszero(delegatecall(gas, caller, add(ins,31), 101, retLoc, retSize)) {
                mstore(0xd,add(2200,mload(0x80)))
                revert(0xd,0x20)
            }
        }
        // End procedure call
        // Being our call to FifthNestedCall
        reqProc = bytes24("FifthNestedCall");
        assembly {
            function malloc(size) -> result {
                // align to 32-byte words
                let rsize := add(size,sub(32,mod(size,32)))
                // get the current free mem location
                result :=  mload(0x40)
                // Bump the value of 0x40 so that it holds the next
                // available memory location.
                mstore(0x40,add(result,rsize))
            }
            let ins := malloc(128)
            // First set up the input data (at memory location 0x0)
            // The call call is 0x-03
            mstore(add(ins,0x0),0x03)
            // The capability index is 0x-02
            mstore(add(ins,0x20),0x02)
            // The key of the procedure
            mstore(add(ins,0x40),reqProc)
            // The size of the return value we expect (0x20)
            let retSize := 0x20
            let retLoc := malloc(retSize)
            mstore(add(ins,0x60),retSize)
            mstore(add(ins,0x80),keccak256(add(fselector,0x20),mload(fselector)))
            // "in_offset" is at 31, because we only want the last byte of type
            // "in_size" is 65 because it is 1+32+32+32+4
            // we will store the result at 0x80 and it will be 32 bytes
            if iszero(delegatecall(gas, caller, add(ins,31), 101, retLoc, retSize)) {
                mstore(0xd,add(2200,mload(0x80)))
                revert(0xd,0x20)
            }
        }
        // End procedure call
        // TODO: perform some checks and return
    }
}