
target/thumbv7m-none-eabi/release/examples/lockopt:     file format elf32-littlearm


Disassembly of section .text:

000000f0 <GPIOA>:
  f0:	21a0      	movs	r1, #160	; 0xa0
  f2:	f3ef 8011 	mrs	r0, BASEPRI
  f6:	f381 8811 	msr	BASEPRI, r1
  fa:	f240 0100 	movw	r1, #0
  fe:	f2c2 0100 	movt	r1, #8192	; 0x2000
 102:	680a      	ldr	r2, [r1, #0]
 104:	3201      	adds	r2, #1
 106:	600a      	str	r2, [r1, #0]
 108:	f24e 2100 	movw	r1, #57856	; 0xe200
 10c:	2202      	movs	r2, #2
 10e:	f2ce 0100 	movt	r1, #57344	; 0xe000
 112:	600a      	str	r2, [r1, #0]
 114:	2204      	movs	r2, #4
 116:	600a      	str	r2, [r1, #0]
 118:	2126      	movs	r1, #38	; 0x26
 11a:	f380 8811 	msr	BASEPRI, r0
 11e:	f2c0 0102 	movt	r1, #2
 122:	2018      	movs	r0, #24
 124:	f000 b887 	b.w	236 <__syscall>

00000128 <GPIOB>:
 128:	21a0      	movs	r1, #160	; 0xa0
 12a:	f3ef 8011 	mrs	r0, BASEPRI
 12e:	f381 8811 	msr	BASEPRI, r1
 132:	f240 0100 	movw	r1, #0
 136:	f2c2 0100 	movt	r1, #8192	; 0x2000
 13a:	680a      	ldr	r2, [r1, #0]
 13c:	3201      	adds	r2, #1
 13e:	600a      	str	r2, [r1, #0]
 140:	f380 8811 	msr	BASEPRI, r0
 144:	4770      	bx	lr

00000146 <GPIOC>:
 146:	f240 0000 	movw	r0, #0
 14a:	f2c2 0000 	movt	r0, #8192	; 0x2000
 14e:	6801      	ldr	r1, [r0, #0]
 150:	3102      	adds	r1, #2
 152:	6001      	str	r1, [r0, #0]
 154:	4770      	bx	lr

00000156 <main>:
 156:	f24e 1000 	movw	r0, #57600	; 0xe100
 15a:	f24e 4201 	movw	r2, #58369	; 0xe401
 15e:	f2ce 0000 	movt	r0, #57344	; 0xe000
 162:	21e0      	movs	r1, #224	; 0xe0
 164:	b672      	cpsid	i
 166:	f880 1300 	strb.w	r1, [r0, #768]	; 0x300
 16a:	2101      	movs	r1, #1
 16c:	f2ce 0200 	movt	r2, #57344	; 0xe000
 170:	23c0      	movs	r3, #192	; 0xc0
 172:	6001      	str	r1, [r0, #0]
 174:	7013      	strb	r3, [r2, #0]
 176:	2302      	movs	r3, #2
 178:	6003      	str	r3, [r0, #0]
 17a:	23a0      	movs	r3, #160	; 0xa0
 17c:	7053      	strb	r3, [r2, #1]
 17e:	2204      	movs	r2, #4
 180:	6002      	str	r2, [r0, #0]
 182:	f64e 5210 	movw	r2, #60688	; 0xed10
 186:	f2ce 0200 	movt	r2, #57344	; 0xe000
 18a:	6813      	ldr	r3, [r2, #0]
 18c:	f043 0302 	orr.w	r3, r3, #2
 190:	6013      	str	r3, [r2, #0]
 192:	f8c0 1100 	str.w	r1, [r0, #256]	; 0x100
 196:	b662      	cpsie	i
 198:	bf30      	wfi
 19a:	e7fd      	b.n	198 <main+0x42>

0000019c <Reset>:
 19c:	f000 f84a 	bl	234 <DefaultPreInit>
 1a0:	f240 0004 	movw	r0, #4
 1a4:	f240 0100 	movw	r1, #0
 1a8:	f2c2 0000 	movt	r0, #8192	; 0x2000
 1ac:	f2c2 0100 	movt	r1, #8192	; 0x2000
 1b0:	4281      	cmp	r1, r0
 1b2:	d214      	bcs.n	1de <Reset+0x42>
 1b4:	f240 0100 	movw	r1, #0
 1b8:	2200      	movs	r2, #0
 1ba:	f2c2 0100 	movt	r1, #8192	; 0x2000
 1be:	f841 2b04 	str.w	r2, [r1], #4
 1c2:	4281      	cmp	r1, r0
 1c4:	bf3c      	itt	cc
 1c6:	f841 2b04 	strcc.w	r2, [r1], #4
 1ca:	4281      	cmpcc	r1, r0
 1cc:	d207      	bcs.n	1de <Reset+0x42>
 1ce:	f841 2b04 	str.w	r2, [r1], #4
 1d2:	4281      	cmp	r1, r0
 1d4:	d203      	bcs.n	1de <Reset+0x42>
 1d6:	f841 2b04 	str.w	r2, [r1], #4
 1da:	4281      	cmp	r1, r0
 1dc:	d3ef      	bcc.n	1be <Reset+0x22>
 1de:	f240 0000 	movw	r0, #0
 1e2:	f240 0100 	movw	r1, #0
 1e6:	f2c2 0000 	movt	r0, #8192	; 0x2000
 1ea:	f2c2 0100 	movt	r1, #8192	; 0x2000
 1ee:	4281      	cmp	r1, r0
 1f0:	d21c      	bcs.n	22c <Reset+0x90>
 1f2:	f240 2150 	movw	r1, #592	; 0x250
 1f6:	f240 0200 	movw	r2, #0
 1fa:	f2c0 0100 	movt	r1, #0
 1fe:	f2c2 0200 	movt	r2, #8192	; 0x2000
 202:	680b      	ldr	r3, [r1, #0]
 204:	f842 3b04 	str.w	r3, [r2], #4
 208:	4282      	cmp	r2, r0
 20a:	d20f      	bcs.n	22c <Reset+0x90>
 20c:	684b      	ldr	r3, [r1, #4]
 20e:	f842 3b04 	str.w	r3, [r2], #4
 212:	4282      	cmp	r2, r0
 214:	bf3e      	ittt	cc
 216:	688b      	ldrcc	r3, [r1, #8]
 218:	f842 3b04 	strcc.w	r3, [r2], #4
 21c:	4282      	cmpcc	r2, r0
 21e:	d205      	bcs.n	22c <Reset+0x90>
 220:	68cb      	ldr	r3, [r1, #12]
 222:	3110      	adds	r1, #16
 224:	f842 3b04 	str.w	r3, [r2], #4
 228:	4282      	cmp	r2, r0
 22a:	d3ea      	bcc.n	202 <Reset+0x66>
 22c:	f7ff ff93 	bl	156 <main>
 230:	defe      	udf	#254	; 0xfe

00000232 <DefaultHandler_>:
 232:	Address 0x0000000000000232 is out of bounds.


00000233 <ADC0_SEQUENCE_0>:
 233:	Address 0x0000000000000233 is out of bounds.


00000234 <DefaultPreInit>:
 234:	Address 0x0000000000000234 is out of bounds.


00000235 <__pre_init>:
 235:	Address 0x0000000000000235 is out of bounds.


00000236 <__syscall>:
 236:	beab      	bkpt	0x00ab
 238:	4770      	bx	lr

0000023a <HardFaultTrampoline>:
 23a:	4670      	mov	r0, lr
 23c:	2104      	movs	r1, #4
 23e:	4208      	tst	r0, r1
 240:	d102      	bne.n	248 <HardFaultTrampoline+0xe>
 242:	f3ef 8008 	mrs	r0, MSP
 246:	e002      	b.n	24e <HardFault_>
 248:	f3ef 8009 	mrs	r0, PSP
 24c:	e7ff      	b.n	24e <HardFault_>

0000024e <HardFault_>:
 24e:	Address 0x000000000000024e is out of bounds.


0000024f <HardFault>:
 24f:	Address 0x000000000000024f is out of bounds.

