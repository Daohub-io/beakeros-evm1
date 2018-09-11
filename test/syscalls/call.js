const debug = require('debug')
const assert = require('assert')

const Kernel = artifacts.require('./Kernel.sol')
const abi = require('ethereumjs-abi')

const beakerlib = require("../../beakerlib");

// Valid Contracts
const Valid = {
    Adder: artifacts.require('test/valid/Adder.sol'),
    Multiply: artifacts.require('test/valid/Multiply.sol'),
    Divide: artifacts.require('test/valid/Divide.sol'),
    SysCallTest: artifacts.require('test/valid/SysCallTest.sol'),
    SysCallTestCall: artifacts.require('test/valid/SysCallTestCall.sol'),
}

const TestWrite = artifacts.require('test/TestWrite.sol');

const Invalid = {
    Simple: artifacts.require('test/invalid/Simple.sol')
}

contract('Kernel', function (accounts) {
    describe('Call capability', function () {
        const procName = "SysCallTestCall";
        const bytecode = Valid.SysCallTestCall.bytecode;

        const testProcName = "TestWrite";
        const testBytecode = TestWrite.bytecode;
        describe('A()', function () {
            const functionSpec = "A()";
            it('A() should succeed when given cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap();
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                // This is the procedure that will do the calling
                const tx1 = await kernel.createProcedure(procName, bytecode, capArray);
                // This is the called procedure
                const tx2 = await kernel.createProcedure(testProcName, testBytecode, beakerlib.Cap.toInput([cap1]));

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.executeProcedure.call(procName, functionSpec, "");
                const tx = await kernel.executeProcedure(procName, functionSpec, "");
                assert.equal(valueX.toNumber(), 111111, "should succeed with zero errcode the first time");

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),356, "new value should be 356");
            })
            it('A() should fail when not given cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2]);

                // This is the procedure that will do the calling
                const tx1 = await kernel.createProcedure(procName, bytecode, capArray);
                // This is the called procedure
                const tx2 = await kernel.createProcedure(testProcName, testBytecode, beakerlib.Cap.toInput([cap1]));

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.executeProcedure.call(procName, functionSpec, "");
                const tx = await kernel.executeProcedure(procName, functionSpec, "");
                assert.equal(valueX.toNumber(), 222233, "should succeed with zero errcode the first time");

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),3, "new value should still be 3");
            })
            it('A() should fail when given the wrong cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.LogCap([]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                // This is the procedure that will do the calling
                const tx1 = await kernel.createProcedure(procName, bytecode, capArray);
                // This is the called procedure
                const tx2 = await kernel.createProcedure(testProcName, testBytecode, beakerlib.Cap.toInput([cap1]));

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.executeProcedure.call(procName, functionSpec, "");
                const tx = await kernel.executeProcedure(procName, functionSpec, "");
                assert.equal(valueX.toNumber(), 222233, "should succeed with zero errcode the first time");

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),3, "new value should still be 3");
            })
            it('A() should succeed with a more restriced cap', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap([testProcName]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                // This is the procedure that will do the calling
                const tx1 = await kernel.createProcedure(procName, bytecode, capArray);
                // This is the called procedure
                const tx2 = await kernel.createProcedure(testProcName, testBytecode, beakerlib.Cap.toInput([cap1]));

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.executeProcedure.call(procName, functionSpec, "");
                const tx = await kernel.executeProcedure(procName, functionSpec, "");
                assert.equal(valueX.toNumber(), 111111, "should succeed with zero errcode the first time");

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),356, "new value should be 356");
            })
            it('A() should fail when the given cap is insufficient', async function () {
                // This tests calls a test procedure which changes a storage
                // value in the kernel from 3 to 356.
                const kernel = await Kernel.new();

                const cap1 = new beakerlib.WriteCap(0x8000,2);
                const cap2 = new beakerlib.LogCap([]);
                const cap3 = new beakerlib.CallCap([procName+"abc"]);
                const capArray = beakerlib.Cap.toInput([cap1, cap2, cap3]);

                // This is the procedure that will do the calling
                const tx1 = await kernel.createProcedure(procName, bytecode, capArray);
                // This is the called procedure
                const tx2 = await kernel.createProcedure(testProcName, testBytecode, beakerlib.Cap.toInput([cap1]));

                const originalValue =  await kernel.testGetter.call();
                assert.equal(originalValue.toNumber(), 3, "test incorrectly set up: initial value should be 3");

                const valueX = await kernel.executeProcedure.call(procName, functionSpec, "");
                const tx = await kernel.executeProcedure(procName, functionSpec, "");
                assert.equal(valueX.toNumber(), 222233, "should succeed with zero errcode the first time");

                const newValue =  await kernel.testGetter.call();
                assert.equal(newValue.toNumber(),3, "new value should still be 3");
            })
        })
    })
})