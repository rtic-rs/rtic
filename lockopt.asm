
target/thumbv7m-none-eabi/release/examples/lockopt:     file format elf32-littlearm


Disassembly of section .text:

000000f0 <GPIOA>:
  f0:	b510      	push	{r4, lr}
  f2:	f000 f8b4 	bl	25e <__basepri_r>
  f6:	4604      	mov	r4, r0
  f8:	20a0      	movs	r0, #160	; 0xa0
  fa:	f000 f8b3 	bl	264 <__basepri_w>
  fe:	f240 0000 	movw	r0, #0
 102:	f2c2 0000 	movt	r0, #8192	; 0x2000
 106:	6801      	ldr	r1, [r0, #0]
 108:	3101      	adds	r1, #1
 10a:	6001      	str	r1, [r0, #0]
 10c:	f24e 2000 	movw	r0, #57856	; 0xe200
 110:	2102      	movs	r1, #2
 112:	f2ce 0000 	movt	r0, #57344	; 0xe000
 116:	6001      	str	r1, [r0, #0]
 118:	2104      	movs	r1, #4
 11a:	6001      	str	r1, [r0, #0]
 11c:	4620      	mov	r0, r4
 11e:	f000 f8a1 	bl	264 <__basepri_w>
 122:	2126      	movs	r1, #38	; 0x26
 124:	2018      	movs	r0, #24
 126:	f2c0 0102 	movt	r1, #2
 12a:	e8bd 4010 	ldmia.w	sp!, {r4, lr}
 12e:	f000 b88e 	b.w	24e <__syscall>

00000132 <GPIOB>:
 132:	b510      	push	{r4, lr}
 134:	f000 f893 	bl	25e <__basepri_r>
 138:	4604      	mov	r4, r0
 13a:	20a0      	movs	r0, #160	; 0xa0
 13c:	f000 f892 	bl	264 <__basepri_w>
 140:	f240 0000 	movw	r0, #0
 144:	f2c2 0000 	movt	r0, #8192	; 0x2000
 148:	6801      	ldr	r1, [r0, #0]
 14a:	3101      	adds	r1, #1
 14c:	6001      	str	r1, [r0, #0]
 14e:	4620      	mov	r0, r4
 150:	e8bd 4010 	ldmia.w	sp!, {r4, lr}
 154:	f000 b886 	b.w	264 <__basepri_w>

00000158 <GPIOC>:
 158:	f240 0000 	movw	r0, #0
 15c:	f2c2 0000 	movt	r0, #8192	; 0x2000
 160:	6801      	ldr	r1, [r0, #0]
 162:	3102      	adds	r1, #2
 164:	6001      	str	r1, [r0, #0]
 166:	4770      	bx	lr

00000168 <main>:
 168:	f000 f873 	bl	252 <__cpsid>
 16c:	f24e 1000 	movw	r0, #57600	; 0xe100
 170:	f24e 4201 	movw	r2, #58369	; 0xe401
 174:	f2ce 0000 	movt	r0, #57344	; 0xe000
 178:	21e0      	movs	r1, #224	; 0xe0
 17a:	f880 1300 	strb.w	r1, [r0, #768]	; 0x300
 17e:	2101      	movs	r1, #1
 180:	f2ce 0200 	movt	r2, #57344	; 0xe000
 184:	23c0      	movs	r3, #192	; 0xc0
 186:	6001      	str	r1, [r0, #0]
 188:	7013      	strb	r3, [r2, #0]
 18a:	2302      	movs	r3, #2
 18c:	6003      	str	r3, [r0, #0]
 18e:	23a0      	movs	r3, #160	; 0xa0
 190:	7053      	strb	r3, [r2, #1]
 192:	2204      	movs	r2, #4
 194:	6002      	str	r2, [r0, #0]
 196:	f64e 5210 	movw	r2, #60688	; 0xed10
 19a:	f2ce 0200 	movt	r2, #57344	; 0xe000
 19e:	6813      	ldr	r3, [r2, #0]
 1a0:	f043 0302 	orr.w	r3, r3, #2
 1a4:	6013      	str	r3, [r2, #0]
 1a6:	f8c0 1100 	str.w	r1, [r0, #256]	; 0x100
 1aa:	f000 f854 	bl	256 <__cpsie>
 1ae:	f000 f854 	bl	25a <__wfi>
 1b2:	e7fc      	b.n	1ae <main+0x46>

000001b4 <Reset>:
 1b4:	f000 f84a 	bl	24c <DefaultPreInit>
 1b8:	f240 0004 	movw	r0, #4
 1bc:	f240 0100 	movw	r1, #0
 1c0:	f2c2 0000 	movt	r0, #8192	; 0x2000
 1c4:	f2c2 0100 	movt	r1, #8192	; 0x2000
 1c8:	4281      	cmp	r1, r0
 1ca:	d214      	bcs.n	1f6 <Reset+0x42>
 1cc:	f240 0100 	movw	r1, #0
 1d0:	2200      	movs	r2, #0
 1d2:	f2c2 0100 	movt	r1, #8192	; 0x2000
 1d6:	f841 2b04 	str.w	r2, [r1], #4
 1da:	4281      	cmp	r1, r0
 1dc:	bf3c      	itt	cc
 1de:	f841 2b04 	strcc.w	r2, [r1], #4
 1e2:	4281      	cmpcc	r1, r0
 1e4:	d207      	bcs.n	1f6 <Reset+0x42>
 1e6:	f841 2b04 	str.w	r2, [r1], #4
 1ea:	4281      	cmp	r1, r0
 1ec:	d203      	bcs.n	1f6 <Reset+0x42>
 1ee:	f841 2b04 	str.w	r2, [r1], #4
 1f2:	4281      	cmp	r1, r0
 1f4:	d3ef      	bcc.n	1d6 <Reset+0x22>
 1f6:	f240 0000 	movw	r0, #0
 1fa:	f240 0100 	movw	r1, #0
 1fe:	f2c2 0000 	movt	r0, #8192	; 0x2000
 202:	f2c2 0100 	movt	r1, #8192	; 0x2000
 206:	4281      	cmp	r1, r0
 208:	d21c      	bcs.n	244 <Reset+0x90>
 20a:	f240 2180 	movw	r1, #640	; 0x280
 20e:	f240 0200 	movw	r2, #0
 212:	f2c0 0100 	movt	r1, #0
 216:	f2c2 0200 	movt	r2, #8192	; 0x2000
 21a:	680b      	ldr	r3, [r1, #0]
 21c:	f842 3b04 	str.w	r3, [r2], #4
 220:	4282      	cmp	r2, r0
 222:	d20f      	bcs.n	244 <Reset+0x90>
 224:	684b      	ldr	r3, [r1, #4]
 226:	f842 3b04 	str.w	r3, [r2], #4
 22a:	4282      	cmp	r2, r0
 22c:	bf3e      	ittt	cc
 22e:	688b      	ldrcc	r3, [r1, #8]
 230:	f842 3b04 	strcc.w	r3, [r2], #4
 234:	4282      	cmpcc	r2, r0
 236:	d205      	bcs.n	244 <Reset+0x90>
 238:	68cb      	ldr	r3, [r1, #12]
 23a:	3110      	adds	r1, #16
 23c:	f842 3b04 	str.w	r3, [r2], #4
 240:	4282      	cmp	r2, r0
 242:	d3ea      	bcc.n	21a <Reset+0x66>
 244:	f7ff ff90 	bl	168 <main>
 248:	defe      	udf	#254	; 0xfe

0000024a <DefaultHandler_>:
 24a:	Address 0x000000000000024a is out of bounds.


0000024b <ADC0_SEQUENCE_0>:
 24b:	Address 0x000000000000024b is out of bounds.


0000024c <DefaultPreInit>:
 24c:	Address 0x000000000000024c is out of bounds.


0000024d <__pre_init>:
 24d:	Address 0x000000000000024d is out of bounds.


0000024e <__syscall>:
 24e:	beab      	bkpt	0x00ab
 250:	4770      	bx	lr

00000252 <__cpsid>:
 252:	b672      	cpsid	i
 254:	4770      	bx	lr

00000256 <__cpsie>:
 256:	b662      	cpsie	i
 258:	4770      	bx	lr

0000025a <__wfi>:
 25a:	bf30      	wfi
 25c:	4770      	bx	lr

0000025e <__basepri_r>:
 25e:	f3ef 8011 	mrs	r0, BASEPRI
 262:	4770      	bx	lr

00000264 <__basepri_w>:
 264:	f380 8811 	msr	BASEPRI, r0
 268:	4770      	bx	lr

0000026a <HardFaultTrampoline>:
 26a:	4670      	mov	r0, lr
 26c:	2104      	movs	r1, #4
 26e:	4208      	tst	r0, r1
 270:	d102      	bne.n	278 <HardFaultTrampoline+0xe>
 272:	f3ef 8008 	mrs	r0, MSP
 276:	e002      	b.n	27e <HardFault_>
 278:	f3ef 8009 	mrs	r0, PSP
 27c:	e7ff      	b.n	27e <HardFault_>

0000027e <HardFault_>:
 27e:	Address 0x000000000000027e is out of bounds.


0000027f <HardFault>:
 27f:	Address 0x000000000000027f is out of bounds.

